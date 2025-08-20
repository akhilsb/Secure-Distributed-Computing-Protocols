use aes::{cipher::{generic_array::GenericArray, KeyInit, BlockEncrypt}, Aes128Enc};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Bencher};
use sha2::{Sha256, Digest};


fn do_hash(one:[u8;32],two:[u8;32], ciph0: &Aes128Enc,ciph1: &Aes128Enc,ciph2: &Aes128Enc)->[u8;32]{
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

fn do_hash_sha256(one:[u8;32],two:[u8;32])->[u8;32]{
    let mut sha256 = Sha256::new();
    sha256.update(one);
    sha256.update(two);
    sha256.finalize().into()
}

fn do_hash_w_aes(one:[u8;32],two:[u8;32])->[u8;32]{
    // let key0 = GenericArray::from([5u8; 16]);
    // let key1 = GenericArray::from([29u8; 16]);
    // let key2 = GenericArray::from([23u8; 16]);

    // let _ciph0 = Aes128Enc::new(&key0);
    // let _ciph1 = Aes128Enc::new(&key1);
    // let _ciph2 = Aes128Enc::new(&key2);
    
    let mut x_11 = [0u8;16];
    let mut x_12 = [0u8;16];
    for i in 0..16{
        x_11[i] = one[i]+2*two[i];
        x_12[i] = one[16+i]+2*two[16+i];
    }
    let blk_11 = GenericArray::from(x_11);
    let blk_12 = GenericArray::from(x_12);
    //ciph0.encrypt_blocks(&mut [blk_11,blk_12]);

    let mut x_21 = [0u8;16];
    let mut x_22 = [0u8;16];
    for i in 0..16{
        x_21[i] = 2*one[i]+2*two[i]+blk_11[i];
        x_22[i] = 2*one[16+i]+2*two[16+i]+blk_12[i];
    }
    let blk_21 = GenericArray::from(x_21);
    let blk_22 = GenericArray::from(x_22);
    //ciph1.encrypt_blocks(&mut [blk_21,blk_22]);

    let mut x_31 = [0u8;16];
    let mut x_32 = [0u8;16];
    
    for i in 0..16{
        x_31[i] = 2*one[i]+two[i]+blk_21[i];
        x_32[i] = 2*one[16+i]+two[16+i]+blk_22[i];
    }
    let blk_31 = GenericArray::from(x_31);
    let blk_32 = GenericArray::from(x_32);
    //ciph2.encrypt_blocks(&mut [blk_31,blk_32]);

    let mut w_1 = [0u8;32];
    for i in 0..16{
        w_1[i] = one[i]+blk_11[i]+blk_21[i]+2*blk_31[i];
    }
    for i in 0..16{
        w_1[16+i] = one[16+i]+blk_12[i]+blk_22[i]+2*blk_32[i];
    }
    return w_1;
}

fn criterion_benchmark(c: &mut Criterion) {
    //let mut group = c.benchmark_group("aes_hash");
    let key0 = GenericArray::from([5u8; 16]);
    let key1 = GenericArray::from([29u8; 16]);
    let key2 = GenericArray::from([23u8; 16]);

    let ciph0 = Aes128Enc::new(&key0);
    let ciph1 = Aes128Enc::new(&key1);
    let ciph2 = Aes128Enc::new(&key2);
    let ciph_tup = ([0u8;32],[10u8;32],ciph0,ciph1,ciph2);

    c.bench_function_over_inputs("aes_hash", 
    |b: &mut Bencher,ciph_tup: &([u8;32],[u8;32],Aes128Enc, Aes128Enc, Aes128Enc)|{
        b.iter(|| do_hash(ciph_tup.0, ciph_tup.1, &ciph_tup.2, &ciph_tup.3, &ciph_tup.4))
    }, vec![ciph_tup]);
    //c.bench_function("aes_hash", &ciph_tup,
    //move |b,(| b.iter(|ciph0,ciph1,ciph2|do_hash(black_box([0u8;32]),black_box([10u8;32]),&ciph0,&ciph1,&ciph2)));
    c.bench_function("aes_hash_w", |b| b.iter(|| do_hash_w_aes(black_box([0u8;32]),black_box([10u8;32]))));
    c.bench_function("sha_hash", |b| b.iter(|| do_hash_sha256(black_box([0u8;32]),black_box([10u8;32]))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);