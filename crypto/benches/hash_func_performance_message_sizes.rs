use aes::{cipher::{generic_array::GenericArray, BlockEncrypt}, Aes128Enc};
use aes_gcm::KeyInit;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crypto::{aes_hash::{HashState, MerkleTree, HASH_SIZE}, hash::Hash};
use sha2::{Digest, Sha256};

fn do_hash(hash_vec: Vec<u8>)->[u8;32]{
    let mut sha256 = Sha256::new();
    sha256.update(hash_vec.as_slice());
    sha256.finalize().into()
}

fn do_hash_aes(bytes: &[u8]) -> Hash{
    let key0 = GenericArray::from([5u8; 16]);
    let key1 = GenericArray::from([29u8; 16]);
    let key2 = GenericArray::from([23u8; 16]);

    let ciph0 = Aes128Enc::new(&key0);
    let ciph1 = Aes128Enc::new(&key1);
    let ciph2 = Aes128Enc::new(&key2);

    let hash_state = HashState{
        aes0: ciph0,
        aes1: ciph1,
        aes2: ciph2
    };

    // Append zeros to make the input a multiple of 32 bytes

    // Divide the input into a vector of 32 byte chunks
    if bytes.len() < 2*HASH_SIZE{
        let mut vec = bytes.to_vec();
        // Append zeros to make the input a multiple of 32 bytes
        let remainder = vec.len() % 64;
        println!("Remainder is {}", remainder);
        if remainder != 0 {
            let padding_needed = 64 - remainder;
            vec.extend(vec![0u8; padding_needed]);
        }
        println!("Padded length is {}", vec.len());
        let mut slice1: Hash = [0u8;32];
        let mut slice2: Hash = [0u8;32];

        slice1.copy_from_slice(&vec[0..32]);
        slice2.copy_from_slice(&vec[32..64]);
        return do_hash_aes_two(slice1, slice2, &hash_state.aes0, &hash_state.aes1, &hash_state.aes2);
    }
    else{
        let chunks: Vec<Hash> = bytes.chunks(HASH_SIZE).map(|x|{
            let mut hash = [0u8;HASH_SIZE];
            hash[0..x.len()].copy_from_slice(x);
            return hash;
        }).collect();
        return MerkleTree::new(chunks, &hash_state).root();
    }
    
}

fn do_hash_aes_two(one:[u8;32],two:[u8;32], ciph0: &Aes128Enc,ciph1: &Aes128Enc,ciph2: &Aes128Enc)->[u8;32]{
    let mut x_11 = [0u8;16];
    let mut x_12 = [0u8;16];
    for i in 0..16{
        x_11[i] = one[i]+2*two[i];
        x_12[i] = one[16+i]+2*two[16+i];
    }
    let blk_11 = GenericArray::from(x_11);
    let blk_12 = GenericArray::from(x_12);
    ciph0.encrypt_blocks(&mut [blk_11,blk_12]);

    let mut x_21 = [0u8;16];
    let mut x_22 = [0u8;16];
    for i in 0..16{
        x_21[i] = 2*one[i]+2*two[i]+blk_11[i];
        x_22[i] = 2*one[16+i]+2*two[16+i]+blk_12[i];
    }
    let blk_21 = GenericArray::from(x_21);
    let blk_22 = GenericArray::from(x_22);
    ciph1.encrypt_blocks(&mut [blk_21,blk_22]);

    let mut x_31 = [0u8;16];
    let mut x_32 = [0u8;16];
    
    for i in 0..16{
        x_31[i] = 2*one[i]+two[i]+blk_21[i];
        x_32[i] = 2*one[16+i]+two[16+i]+blk_22[i];
    }
    let blk_31 = GenericArray::from(x_31);
    let blk_32 = GenericArray::from(x_32);
    ciph2.encrypt_blocks(&mut [blk_31,blk_32]);

    let mut w_1 = [0u8;32];
    for i in 0..16{
        w_1[i] = one[i]+blk_11[i]+blk_21[i]+2*blk_31[i];
    }
    for i in 0..16{
        w_1[16+i] = one[16+i]+blk_12[i]+blk_22[i]+2*blk_32[i];
    }
    return w_1;
}


fn hash_func_benchmark(c : &mut Criterion){
    c.bench_function("sha_hash_10kb", |b| b.iter(|| do_hash(black_box([0u8;10240].to_vec()))));
    c.bench_function("sha_hash_100kb", |b| b.iter(|| do_hash(black_box([0u8;102400].to_vec()))));
    c.bench_function("sha_hash_1mb", |b| b.iter(|| do_hash(black_box([0u8;1024000].to_vec()))));

    c.bench_function("aes_hash_10kb", |b| b.iter(|| do_hash_aes(black_box(&[0u8;11]))));
    c.bench_function("aes_hash_100kb", |b| b.iter(|| do_hash_aes(black_box(&[0u8;102400]))));
    c.bench_function("aes_hash_1mb", |b| b.iter(|| do_hash_aes(black_box(&[0u8;1024000]))));
}

criterion_group!(benches, hash_func_benchmark);
criterion_main!(benches);