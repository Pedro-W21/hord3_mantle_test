
#[test]
pub fn test_turboset() {
    use crate::game_map::turbo_set::TurboSet;

    let mut turboset = TurboSet::new(10000, 100000);
    
    for i in 0..10000 {
        let target_id = fastrand::usize(0..10000);
        if fastrand::bool() {
            turboset.insert(target_id);
            dbg!(turboset.get_data());
            assert!(turboset.contains(target_id));
        }
        else {
            turboset.remove(target_id);
            dbg!(turboset.get_data());
            assert!(!turboset.contains(target_id));
        }
    }
}


#[test]
pub fn test_turboset_iter() {
    use crate::game_map::turbo_set::TurboSet;

    for i in 0..100 {
        let mut turboset = TurboSet::new(10000, 100000);
        let mut added = Vec::with_capacity(100);
        for i in 0..10 {
            let target_id = fastrand::usize(0..10000);
            if !added.contains(&target_id) {
                added.push(target_id);
                turboset.insert(target_id);
            }
        }
        added.sort();

        let iterator = turboset.iter();
        let retrieved:Vec<usize> = iterator.collect();

        assert_eq!(added, retrieved)
    }
    
}