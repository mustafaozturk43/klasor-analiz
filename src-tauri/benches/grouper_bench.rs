//! Similar kumeleme performans karsilastirmasi: naif (tum ciftler) vs optimize.
//! Calistir: cargo bench --bench grouper_bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use smart_file_organizer_lib::models::{FileCategory, FileItem};
use smart_file_organizer_lib::similarity;

#[path = "../tests/support/naive.rs"]
mod naive;

/// 20 farkli onek (bloklama) ve aile ici benzer stem'ler uretir.
const PREFIXES: &[&str] = &[
    "rapor", "fatura", "resim", "belge", "not", "arsiv", "tatil", "video", "muzik", "sunum",
    "veri", "yedek", "proje", "test", "ornek", "kayit", "ozel", "genel", "yeni", "eski",
];

fn make_items(n: usize) -> Vec<FileItem> {
    (0..n)
        .map(|i| {
            let stem = format!("{}_dosya_{:04}", PREFIXES[i % PREFIXES.len()], i);
            FileItem {
                path: format!("C:\\x\\{i}.txt"),
                name: format!("{i}.txt"),
                stem,
                extension: "txt".to_string(),
                size: 100,
                mtime: 0,
                category: FileCategory::Document,
                hash_id: String::new(),
            }
        })
        .collect()
}

fn bench_cluster(c: &mut Criterion) {
    for n in [500usize, 2000] {
        let items = make_items(n);

        c.bench_function(&format!("naive_{n}"), |b| {
            b.iter(|| naive::cluster_naive(black_box(items.clone()), black_box(0.82)))
        });

        c.bench_function(&format!("optimized_{n}"), |b| {
            b.iter(|| {
                similarity::cluster_by_similarity(
                    black_box(items.clone()),
                    black_box(0.82),
                    &|| false,
                )
            })
        });
    }
}

criterion_group!(benches, bench_cluster);
criterion_main!(benches);
