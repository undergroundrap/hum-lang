#[cfg(test)]
use hum_sha256::{checked_bit_length, digest_checked};
pub(crate) use hum_sha256::{digest, lowercase_hex};

#[cfg(test)]
mod tests {
    use super::{checked_bit_length, digest, digest_checked, lowercase_hex};

    #[test]
    fn sha256_known_answer_and_boundary_matrix_is_exact() {
        let vectors = [
            (
                Vec::new(),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc".to_vec(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
            (
                b"hello world".to_vec(),
                "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
            ),
            (
                vec![b'a'; 55],
                "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
            ),
            (
                vec![b'a'; 56],
                "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
            ),
            (
                vec![b'a'; 63],
                "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
            ),
            (
                vec![b'a'; 64],
                "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
            ),
            (
                vec![b'a'; 65],
                "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
            ),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".to_vec(),
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            ),
            (
                vec![b'a'; 1_000_000],
                "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
            ),
            (
                include_bytes!("../examples/core/minimal_add.hum").to_vec(),
                "aeae6ae9de975eee9873c3d9ece891e66bd7d6881b5035c24b1a11f3902a52b6",
            ),
        ];
        for (input, expected) in vectors {
            assert_eq!(
                lowercase_hex(&digest(&input).expect("bounded input")),
                expected
            );
        }
        assert_eq!(
            include_bytes!("../examples/core/minimal_add.hum").len(),
            121
        );
        assert!(digest_checked(&[]).is_ok());
        assert!(checked_bit_length(u64::MAX / 8).is_ok());
        assert!(checked_bit_length(u64::MAX / 8 + 1).is_err());
        let artifact = crate::backend_input::minimal_add_artifact_for_test();
        assert_eq!(
            lowercase_hex(&digest(artifact.payload()).expect("bounded payload")),
            artifact.artifact_id().trim_start_matches("sha256:")
        );
    }
}
