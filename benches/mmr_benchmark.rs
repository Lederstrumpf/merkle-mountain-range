#[macro_use]
extern crate criterion;

use criterion::{black_box, BenchmarkId, Criterion};

use bytes::Bytes;
use polkadot_ckb_merkle_mountain_range::ancestry_proof::expected_ancestry_proof_size;
use polkadot_ckb_merkle_mountain_range::{
    util::MemStore, Error, MMRStoreReadOps, Merge, Result, MMR,
};
use rand::{seq::SliceRandom, thread_rng};
use std::convert::TryFrom;

use blake2b_rs::{Blake2b, Blake2bBuilder};
use itertools::iproduct;

fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32).build()
}

#[derive(Eq, PartialEq, Clone, Debug, Default)]
struct NumberHash(pub Bytes);
impl TryFrom<u32> for NumberHash {
    type Error = Error;
    fn try_from(num: u32) -> Result<Self> {
        let mut hasher = new_blake2b();
        let mut hash = [0u8; 32];
        hasher.update(&num.to_le_bytes());
        hasher.finalize(&mut hash);
        Ok(NumberHash(hash.to_vec().into()))
    }
}

struct MergeNumberHash;

impl Merge for MergeNumberHash {
    type Item = NumberHash;
    fn merge(lhs: &Self::Item, rhs: &Self::Item) -> Result<Self::Item> {
        let mut hasher = new_blake2b();
        let mut hash = [0u8; 32];
        hasher.update(&lhs.0);
        hasher.update(&rhs.0);
        hasher.finalize(&mut hash);
        Ok(NumberHash(hash.to_vec().into()))
    }
}

fn prepare_mmr(count: u32) -> (u64, MemStore<NumberHash>, Vec<u64>, Vec<(u32, NumberHash)>) {
    let store = MemStore::default();
    let mut prev_roots = Vec::new();
    let mut mmr = MMR::<_, MergeNumberHash, _>::new(0, &store);
    let positions: Vec<u64> = (0u32..count)
        .map(|i| {
            let position = mmr.push(NumberHash::try_from(i).unwrap()).unwrap();
            prev_roots.push((i + 1, mmr.get_root().expect("get root")));
            position
        })
        .collect();
    let mmr_size = mmr.mmr_size();
    mmr.commit().expect("write to store");
    (mmr_size, store, positions, prev_roots)
}

const INDEX_OFFSET: u64 = 100_000;
const INDEX_DOMAIN: u64 = 2_000;

fn bench(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("MMR insertion");
        let inputs = [10_000, 100_000, 100_0000];
        for input in inputs.iter() {
            group.bench_with_input(BenchmarkId::new("times", input), &input, |b, &&size| {
                b.iter(|| prepare_mmr(size));
            });
        }
    }

    c.bench_function("MMR gen proof", |b| {
        let (mmr_size, store, positions, _) = prepare_mmr(1000_000);
        let mmr = MMR::<_, MergeNumberHash, _>::new(mmr_size, &store);
        let mut rng = thread_rng();
        b.iter(|| mmr.gen_proof(vec![*positions.choose(&mut rng).unwrap()]));
    });

    c.bench_function("MMR gen node-proof", |b| {
        let (mmr_size, store, positions, _) = prepare_mmr(1000_000);
        let mmr = MMR::<_, MergeNumberHash, _>::new(mmr_size, &store);
        let mut rng = thread_rng();
        b.iter(|| mmr.gen_node_proof(vec![*positions.choose(&mut rng).unwrap()]));
    });

    c.bench_function("MMR gen ancestry-proof", |b| {
        let (mmr_size, store, _positions, roots) = prepare_mmr(1000_000);
        let mmr = MMR::<_, MergeNumberHash, _>::new(mmr_size, &store);
        let mut rng = thread_rng();
        b.iter(|| mmr.gen_ancestry_proof(roots.choose(&mut rng).unwrap().0 as u64));
    });

    c.bench_function("MMR verify", |b| {
        let (mmr_size, store, positions, _) = prepare_mmr(1000_000);
        let mmr = MMR::<_, MergeNumberHash, _>::new(mmr_size, &store);
        let mut rng = thread_rng();
        let root: NumberHash = mmr.get_root().unwrap();
        let proofs: Vec<_> = (0..10_000)
            .map(|_| {
                let pos = positions.choose(&mut rng).unwrap();
                let elem = (&store).get_elem(*pos).unwrap().unwrap();
                let proof = mmr.gen_proof(vec![*pos]).unwrap();
                (pos, elem, proof)
            })
            .collect();
        b.iter(|| {
            let (pos, elem, proof) = proofs.choose(&mut rng).unwrap();
            proof
                .verify(root.clone(), vec![(**pos, elem.clone())])
                .unwrap();
        });
    });

    c.bench_function("MMR verify node-proof", |b| {
        let (mmr_size, store, positions, _) = prepare_mmr(1000_000);
        let mmr = MMR::<_, MergeNumberHash, _>::new(mmr_size, &store);
        let mut rng = thread_rng();
        let root: NumberHash = mmr.get_root().unwrap();
        let proofs: Vec<_> = (0..10_000)
            .map(|_| {
                let pos = positions.choose(&mut rng).unwrap();
                let elem = (&store).get_elem(*pos).unwrap().unwrap();
                let proof = mmr.gen_node_proof(vec![*pos]).unwrap();
                (pos, elem, proof)
            })
            .collect();
        b.iter(|| {
            let (pos, elem, proof) = proofs.choose(&mut rng).unwrap();
            proof
                .verify(root.clone(), vec![(**pos, elem.clone())])
                .unwrap();
        });
    });

    c.bench_function("MMR verify ancestry-proof", |b| {
        let (mmr_size, store, _positions, roots) = prepare_mmr(1000_000);
        let mmr = MMR::<_, MergeNumberHash, _>::new(mmr_size, &store);
        let mut rng = thread_rng();
        let root: NumberHash = mmr.get_root().unwrap();
        let proofs: Vec<_> = (0..10_000)
            .map(|_| {
                let (prev_size, prev_root) = roots.choose(&mut rng).unwrap();
                let proof = mmr.gen_ancestry_proof(*prev_size as u64).unwrap();
                (prev_root, proof)
            })
            .collect();
        b.iter(|| {
            let (prev_root, proof) = proofs.choose(&mut rng).unwrap();
            proof
                .verify_ancestor(root.clone(), prev_root.clone().clone())
                .unwrap();
        });
    });

    c.bench_function("expected_ancestry_proof_size", |b| {
        b.iter(|| {
            for (i, j) in iproduct!(
                INDEX_OFFSET..INDEX_OFFSET + INDEX_DOMAIN,
                INDEX_OFFSET + 1..INDEX_OFFSET + INDEX_DOMAIN + 1
            ) {
                if i < j {
                    // black_box(expected_ancestry_proof_size(i, j));
                    black_box(expected_ancestry_proof_size(i, j));
                }
            }
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = bench
);
criterion_main!(benches);
