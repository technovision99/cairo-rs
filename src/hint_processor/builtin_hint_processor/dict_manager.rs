use crate::{
    types::relocatable::{MaybeRelocatable, Relocatable},
    vm::{
        errors::{
            hint_errors::HintError, memory_errors::MemoryError, vm_errors::VirtualMachineError,
        },
        vm_core::VirtualMachine,
    },
};
use felt::Felt;
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
///Manages dictionaries in a Cairo program.
///Uses the segment index to associate the corresponding python dict with the Cairo dict.
pub struct DictManager {
    pub trackers: HashMap<isize, DictTracker>,
}

#[derive(PartialEq, Debug, Clone)]
///Tracks the python dict associated with a Cairo dict.
pub struct DictTracker {
    //Dictionary.
    pub data: Dictionary,
    //Pointer to the first unused position in the dict segment.
    pub current_ptr: Relocatable,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Dictionary {
    SimpleDictionary(HashMap<Felt, Felt>),
    DefaultDictionary {
        dict: HashMap<Felt, Felt>,
        default_value: Felt,
    },
}

impl Dictionary {
    fn get(&mut self, key: &Felt) -> Option<&Felt> {
        match self {
            Self::SimpleDictionary(dict) => dict.get(key),
            Self::DefaultDictionary {
                dict,
                default_value,
            } => Some(
                dict.entry(key.clone())
                    .or_insert_with(|| default_value.clone()),
            ),
        }
    }

    fn insert(&mut self, key: &Felt, value: &Felt) {
        let dict = match self {
            Self::SimpleDictionary(dict) => dict,
            Self::DefaultDictionary {
                dict,
                default_value: _,
            } => dict,
        };
        dict.insert(key.clone(), value.clone());
    }
}

impl DictManager {
    pub fn new() -> Self {
        DictManager {
            trackers: HashMap::<isize, DictTracker>::new(),
        }
    }
    //Creates a new Cairo dictionary. The values of initial_dict can be integers, tuples or
    //lists. See MemorySegments.gen_arg().
    //For now, no initial dict will be processed (Assumes initial_dict = None)
    pub fn new_dict(
        &mut self,
        vm: &mut VirtualMachine,
        initial_dict: HashMap<Felt, Felt>,
    ) -> Result<MaybeRelocatable, HintError> {
        let base = vm.add_memory_segment();
        if self.trackers.contains_key(&base.segment_index) {
            return Err(HintError::CantCreateDictionaryOnTakenSegment(
                base.segment_index,
            ));
        }

        if base.segment_index < 0 {
            Err(VirtualMachineError::MemoryError(
                MemoryError::AddressInTemporarySegment(base.segment_index),
            ))?;
        };

        self.trackers.insert(
            base.segment_index,
            DictTracker::new_with_initial(&base, initial_dict),
        );
        Ok(MaybeRelocatable::RelocatableValue(base))
    }

    //Creates a new Cairo default dictionary
    pub fn new_default_dict(
        &mut self,
        vm: &mut VirtualMachine,
        default_value: &Felt,
        initial_dict: Option<HashMap<Felt, Felt>>,
    ) -> Result<MaybeRelocatable, HintError> {
        let base = vm.add_memory_segment();
        if self.trackers.contains_key(&base.segment_index) {
            return Err(HintError::CantCreateDictionaryOnTakenSegment(
                base.segment_index,
            ));
        }
        self.trackers.insert(
            base.segment_index,
            DictTracker::new_default_dict(&base, default_value, initial_dict),
        );
        Ok(MaybeRelocatable::RelocatableValue(base))
    }

    //Returns the tracker which's current_ptr matches with the given dict_ptr
    pub fn get_tracker_mut(
        &mut self,
        dict_ptr: &Relocatable,
    ) -> Result<&mut DictTracker, HintError> {
        let tracker = self
            .trackers
            .get_mut(&dict_ptr.segment_index)
            .ok_or(HintError::NoDictTracker(dict_ptr.segment_index))?;
        if tracker.current_ptr != *dict_ptr {
            return Err(HintError::MismatchedDictPtr(tracker.current_ptr, *dict_ptr));
        }
        Ok(tracker)
    }

    //Returns the tracker which's current_ptr matches with the given dict_ptr
    pub fn get_tracker(&self, dict_ptr: &Relocatable) -> Result<&DictTracker, HintError> {
        let tracker = self
            .trackers
            .get(&dict_ptr.segment_index)
            .ok_or(HintError::NoDictTracker(dict_ptr.segment_index))?;
        if tracker.current_ptr != *dict_ptr {
            return Err(HintError::MismatchedDictPtr(tracker.current_ptr, *dict_ptr));
        }
        Ok(tracker)
    }
}

impl Default for DictManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DictTracker {
    pub fn new_empty(base: &Relocatable) -> Self {
        DictTracker {
            data: Dictionary::SimpleDictionary(HashMap::new()),
            current_ptr: *base,
        }
    }

    pub fn new_default_dict(
        base: &Relocatable,
        default_value: &Felt,
        initial_dict: Option<HashMap<Felt, Felt>>,
    ) -> Self {
        DictTracker {
            data: Dictionary::DefaultDictionary {
                dict: if let Some(dict) = initial_dict {
                    dict
                } else {
                    HashMap::new()
                },
                default_value: default_value.clone(),
            },
            current_ptr: *base,
        }
    }

    pub fn new_with_initial(base: &Relocatable, initial_dict: HashMap<Felt, Felt>) -> Self {
        DictTracker {
            data: Dictionary::SimpleDictionary(initial_dict),
            current_ptr: *base,
        }
    }

    //Returns a copy of the contained dictionary, losing the dictionary type in the process
    pub fn get_dictionary_copy(&self) -> HashMap<Felt, Felt> {
        match &self.data {
            Dictionary::SimpleDictionary(dict) => dict.clone(),
            Dictionary::DefaultDictionary {
                dict,
                default_value: _,
            } => dict.clone(),
        }
    }

    pub fn get_value(&mut self, key: &Felt) -> Result<&Felt, HintError> {
        self.data
            .get(key)
            .ok_or_else(|| HintError::NoValueForKey(key.clone()))
    }

    pub fn insert_value(&mut self, key: &Felt, val: &Felt) {
        self.data.insert(key, val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{relocatable, utils::test_utils::*, vm::vm_core::VirtualMachine};
    use felt::NewFelt;
    use num_traits::One;

    #[test]
    fn create_dict_manager() {
        let dict_manager = DictManager::new();
        assert_eq!(dict_manager.trackers, HashMap::new());
    }

    #[test]
    fn create_dict_tracker_empty() {
        let dict_tracker = DictTracker::new_empty(&relocatable!(1, 0));
        assert_eq!(
            dict_tracker.data,
            Dictionary::SimpleDictionary(HashMap::new())
        );
        assert_eq!(dict_tracker.current_ptr, relocatable!(1, 0));
    }

    #[test]
    fn create_dict_tracker_default() {
        let dict_tracker = DictTracker::new_default_dict(&relocatable!(1, 0), &Felt::new(5), None);
        assert_eq!(
            dict_tracker.data,
            Dictionary::DefaultDictionary {
                dict: HashMap::new(),
                default_value: Felt::new(5)
            }
        );
        assert_eq!(dict_tracker.current_ptr, relocatable!(1, 0));
    }

    #[test]
    fn dict_manager_new_dict_empty() {
        let mut vm = vm!();
        let mut dict_manager = DictManager::new();
        let base = dict_manager.new_dict(&mut vm, HashMap::new());
        assert_eq!(base, Ok(MaybeRelocatable::from((0, 0))));
        assert!(dict_manager.trackers.contains_key(&0));
        assert_eq!(
            dict_manager.trackers.get(&0),
            Some(&DictTracker::new_empty(&relocatable!(0, 0)))
        );
        assert_eq!(vm.segments.num_segments, 1);
    }

    #[test]
    fn dict_manager_new_dict_default() {
        let mut dict_manager = DictManager::new();
        let mut vm = vm!();
        let base = dict_manager.new_default_dict(&mut vm, &Felt::new(5), None);
        assert_eq!(base, Ok(MaybeRelocatable::from((0, 0))));
        assert!(dict_manager.trackers.contains_key(&0));
        assert_eq!(
            dict_manager.trackers.get(&0),
            Some(&DictTracker::new_default_dict(
                &relocatable!(0, 0),
                &Felt::new(5),
                None
            ))
        );
        assert_eq!(vm.segments.num_segments, 1);
    }

    #[test]
    fn dict_manager_new_dict_with_initial_dict() {
        let mut dict_manager = DictManager::new();
        let mut vm = vm!();
        let mut initial_dict = HashMap::<Felt, Felt>::new();
        initial_dict.insert(Felt::new(5), Felt::new(5));
        let base = dict_manager.new_dict(&mut vm, initial_dict.clone());
        assert_eq!(base, Ok(MaybeRelocatable::from((0, 0))));
        assert!(dict_manager.trackers.contains_key(&0));
        assert_eq!(
            dict_manager.trackers.get(&0),
            Some(&DictTracker::new_with_initial(
                &relocatable!(0, 0),
                initial_dict
            ))
        );
        assert_eq!(vm.segments.num_segments, 1);
    }

    #[test]
    fn dict_manager_new_default_dict_with_initial_dict() {
        let mut dict_manager = DictManager::new();
        let mut initial_dict = HashMap::<Felt, Felt>::new();
        let mut vm = vm!();
        initial_dict.insert(Felt::new(5), Felt::new(5));
        let base =
            dict_manager.new_default_dict(&mut vm, &Felt::new(7), Some(initial_dict.clone()));
        assert_eq!(base, Ok(MaybeRelocatable::from((0, 0))));
        assert!(dict_manager.trackers.contains_key(&0));
        assert_eq!(
            dict_manager.trackers.get(&0),
            Some(&DictTracker::new_default_dict(
                &relocatable!(0, 0),
                &Felt::new(7),
                Some(initial_dict)
            ))
        );
        assert_eq!(vm.segments.num_segments, 1);
    }

    #[test]
    fn dict_manager_new_dict_empty_same_segment() {
        let mut dict_manager = DictManager::new();
        dict_manager
            .trackers
            .insert(0, DictTracker::new_empty(&relocatable!(0, 0)));
        let mut vm = vm!();
        assert_eq!(
            dict_manager.new_dict(&mut vm, HashMap::new()),
            Err(HintError::CantCreateDictionaryOnTakenSegment(0))
        );
    }

    #[test]
    fn dict_manager_new_default_dict_empty_same_segment() {
        let mut dict_manager = DictManager::new();
        dict_manager.trackers.insert(
            0,
            DictTracker::new_default_dict(&relocatable!(0, 0), &Felt::new(6), None),
        );
        let mut vm = vm!();
        assert_eq!(
            dict_manager.new_dict(&mut vm, HashMap::new()),
            Err(HintError::CantCreateDictionaryOnTakenSegment(0))
        );
    }

    #[test]
    fn dictionary_get_insert_simple() {
        let mut dictionary = Dictionary::SimpleDictionary(HashMap::new());
        dictionary.insert(&Felt::one(), &Felt::new(2));
        assert_eq!(dictionary.get(&Felt::one()), Some(&Felt::new(2)));
        assert_eq!(dictionary.get(&Felt::new(2)), None);
    }

    #[test]
    fn dictionary_get_insert_default() {
        let mut dictionary = Dictionary::DefaultDictionary {
            dict: HashMap::new(),
            default_value: Felt::new(7),
        };
        dictionary.insert(&Felt::one(), &Felt::new(2));
        assert_eq!(dictionary.get(&Felt::one()), Some(&Felt::new(2)));
        assert_eq!(dictionary.get(&Felt::new(2)), Some(&Felt::new(7)));
    }
}
