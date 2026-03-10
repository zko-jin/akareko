use akareko_lib::{
    db::{
        Repositories,
        user::{I2PAddress, User},
    },
    types::{PublicKey, Signature, String8, Timestamp},
};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::Rng;
use surrealdb::{Surreal, engine::local::SurrealKv};
use tokio::runtime::Builder;

fn rand_user_vec(size: usize) -> Vec<User> {
    fn rand_user() -> User {
        // Randomly generate 32 bytes
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);

        User::new(
            String8::new("Test User".to_string()).unwrap(),
            Timestamp::new(0),
            unsafe { PublicKey::from_bytes_unchecked(bytes) },
            Signature::empty(),
            I2PAddress::new("aksjdpoifqwhpi1209u209dn7u9n13i.b64.i2p"),
        )
    }

    (0..size).map(|_| rand_user()).collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("user");
    group.sample_size(10);

    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    let repo = rt.block_on(async {
        Repositories::setup(
            Surreal::new::<SurrealKv>("benches/db/surreal")
                .await
                .unwrap(),
        )
        .await
    });

    let size = 1000;

    group.bench_function(BenchmarkId::new("db_insert_user", size), |b| {
        b.to_async(&rt).iter_batched(
            || rand_user_vec(size),
            |data| async {
                repo.user().upsert_users(data).await.unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    // group.bench_with_input(BenchmarkId::new("db_insert_", size), &users_vec, |b, v| {
    //     b.to_async(&rt).iter(|| async { for user in v {} });
    // });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
