// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    iterator::JellyfishMerkleIterator, mock_tree_store::MockTreeStore, JellyfishMerkleTree,
};
use crypto::HashValue;
use failure::prelude::*;
use libra_types::{account_state_blob::AccountStateBlob, transaction::Version};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::BTreeMap;

#[test]
fn test_iterator_same_version() {
    for i in (1..100).step_by(11) {
        test_n_leaves_same_version(i);
    }
}

#[test]
fn test_iterator_multiple_versions() {
    test_n_leaves_multiple_versions(50);
}

fn test_n_leaves_same_version(n: usize) {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let mut rng = StdRng::from_seed([1; 32]);

    let mut btree = BTreeMap::new();
    for i in 0..n {
        let key = HashValue::random_with_rng(&mut rng);
        let value = AccountStateBlob::from(i.to_be_bytes().to_vec());
        assert_eq!(btree.insert(key, value), None);
    }

    let (_root_hash, batch) = tree
        .put_blob_set(btree.clone().into_iter().collect(), 0 /* version */)
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();

    run_tests(&db, &btree, 0 /* version */);
}

fn test_n_leaves_multiple_versions(n: usize) {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let mut rng = StdRng::from_seed([1; 32]);

    let mut btree = BTreeMap::new();
    for i in 0..n {
        let key = HashValue::random_with_rng(&mut rng);
        let value = AccountStateBlob::from(i.to_be_bytes().to_vec());
        assert_eq!(btree.insert(key, value.clone()), None);
        let (_root_hash, batch) = tree.put_blob_set(vec![(key, value)], i as Version).unwrap();
        db.write_tree_update_batch(batch).unwrap();
        run_tests(&db, &btree, i as Version);
    }
}

fn run_tests(db: &MockTreeStore, btree: &BTreeMap<HashValue, AccountStateBlob>, version: Version) {
    {
        let iter = JellyfishMerkleIterator::new(db, version, HashValue::zero()).unwrap();
        assert_eq!(
            iter.collect::<Result<Vec<_>>>().unwrap(),
            btree.clone().into_iter().collect::<Vec<_>>(),
        );
    }

    for i in 0..btree.len() {
        let ith_key = *btree.keys().nth(i).unwrap();

        {
            let iter = JellyfishMerkleIterator::new(db, version, ith_key).unwrap();
            assert_eq!(
                iter.collect::<Result<Vec<_>>>().unwrap(),
                btree.clone().into_iter().skip(i).collect::<Vec<_>>(),
            );
        }

        {
            let ith_key_plus_one = plus_one(ith_key);
            let iter = JellyfishMerkleIterator::new(db, version, ith_key_plus_one).unwrap();
            assert_eq!(
                iter.collect::<Result<Vec<_>>>().unwrap(),
                btree.clone().into_iter().skip(i + 1).collect::<Vec<_>>(),
            );
        }
    }

    {
        let iter =
            JellyfishMerkleIterator::new(db, version, HashValue::new([0xFF; HashValue::LENGTH]))
                .unwrap();
        assert_eq!(iter.collect::<Result<Vec<_>>>().unwrap(), vec![]);
    }
}

fn plus_one(hash: HashValue) -> HashValue {
    let mut buf = hash.to_vec();
    for i in (0..HashValue::LENGTH).rev() {
        if buf[i] == 255 {
            buf[i] = 0;
        } else {
            buf[i] += 1;
            break;
        }
    }
    HashValue::from_slice(&buf).unwrap()
}
