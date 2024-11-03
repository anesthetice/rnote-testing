// Imports
use rnote_engine::{
    engine::EngineSnapshot,
    fileformats::{
        rnoteformat::{CompressionMethod, RnoteFile, SerializationMethod},
        FileFormatLoader,
    },
};
use std::{io::Write, time::Instant};

static FILE_LIST: [&str; 11] = [
    "files/1.rnote",
    "files/2.rnote",
    "files/3.rnote",
    "files/4.rnote",
    "files/5.rnote",
    "files/6.rnote",
    "files/7.rnote",
    "files/8.rnote",
    "files/9.rnote",
    "files/10.rnote",
    "files/11.rnote",
];

// repr. by seconds
static BASELINE_TIME_TO_COMPRESS: [f64; 11] = [
    0.0102834658,
    0.0414146274,
    0.07697474420000001,
    0.14195840799999998,
    0.2160995174,
    0.43298379919999996,
    0.5648528078,
    0.7240034852,
    1.0731516998,
    1.4960087022,
    2.5224414204000003,
];

// repr. by MB
static BASELINE_SER_COMP_SIZE: [f64; 11] = [
    0.111546, 0.530866, 1.033512, 1.990903, 3.030683, 6.138827, 8.096551, 10.299053, 15.269708,
    20.885667, 30.338339,
];

/// ## Comparative Compression Ratio
/// baseline compressed size (Json, Gzip5) minus new compressed size
/// divided by baseline compressed size
///
/// ## Comparative Time Ratio
/// baseline time to serialize + compress minus new time
/// divided by baseline time
fn run_test(
    engine_snapshots: Vec<EngineSnapshot>,
    ser_method: SerializationMethod,
    cmp_method: CompressionMethod,
    extra: &str,
) -> anyhow::Result<()> {
    let mut compressed_data_size_mb: [f64; 11] = [0.0; 11];
    let mut time_to_serialize_and_compress_seconds: [f64; 11] = [0.0; 11];

    for (idx, engine_snapshot) in engine_snapshots.into_iter().enumerate() {
        for _ in 0..3 {
            let start = Instant::now();
            let uncompressed_data = ser_method.serialize(&engine_snapshot)?;
            let compressed_data = cmp_method.compress(uncompressed_data)?;
            time_to_serialize_and_compress_seconds[idx] += start.elapsed().as_secs_f64();
            compressed_data_size_mb[idx] = compressed_data.len() as f64 / 1e6;
        }
    }

    for x in time_to_serialize_and_compress_seconds.iter_mut() {
        *x /= 3.0;
    }

    let ccr = compressed_data_size_mb
        .into_iter()
        .zip(BASELINE_SER_COMP_SIZE)
        .map(|(new, base)| (base - new) / (base * 11.0))
        .sum::<f64>();

    let ctr = time_to_serialize_and_compress_seconds
        .into_iter()
        .zip(BASELINE_TIME_TO_COMPRESS)
        .map(|(new, base)| (base - new) / (base * 11.0))
        .sum::<f64>();

    let output = format!(
        "{}{:?}, {:?} => ccr: {:.5} ; ctr: {:.5}\n",
        extra, ser_method, cmp_method, ccr, ctr
    );

    println!("{}", output);

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("./ledger.txt")?
        .write_all(output.as_bytes())?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let engine_snapshots = get_rnote_engine_snapshots();

    run_test(
        engine_snapshots.clone(),
        SerializationMethod::Json,
        CompressionMethod::Zstd(9),
        "",
    )?;

    run_test(
        engine_snapshots.clone(),
        SerializationMethod::Bitcode,
        CompressionMethod::Zstd(9),
        "",
    )?;

    run_test(
        engine_snapshots.clone(),
        SerializationMethod::Flatbuffers,
        CompressionMethod::Zstd(9),
        "",
    )?;

    Ok(())
}

fn get_rnote_engine_snapshots() -> Vec<EngineSnapshot> {
    FILE_LIST
        .iter()
        .map(|filepath| {
            let data = std::fs::read(filepath).unwrap();
            EngineSnapshot::try_from(RnoteFile::load_from_bytes(&data).unwrap()).unwrap()
        })
        .collect()
}

fn establish_baseline(engine_snapshots: Vec<EngineSnapshot>) {
    let baseline_serialization_method = SerializationMethod::Json;
    let baseline_compression_method = CompressionMethod::Gzip(5);

    let mut compressed_data_size_mb: [f64; 11] = [0.0; 11];
    let mut time_to_serialize_and_compress_seconds: [f64; 11] = [0.0; 11];

    for (idx, engine_snapshot) in engine_snapshots.into_iter().enumerate() {
        for _ in 0..5 {
            let start = Instant::now();
            let uncompressed_data = baseline_serialization_method
                .serialize(&engine_snapshot)
                .unwrap();
            let compressed_data = baseline_compression_method
                .compress(uncompressed_data)
                .unwrap();
            time_to_serialize_and_compress_seconds[idx] += start.elapsed().as_secs_f64();
            compressed_data_size_mb[idx] = compressed_data.len() as f64 / 1e6;
        }
    }

    for x in time_to_serialize_and_compress_seconds.iter_mut() {
        *x /= 5.0;
    }

    println!(
        "comp : {:?}\ntime : {:?}",
        compressed_data_size_mb, time_to_serialize_and_compress_seconds
    );
}
