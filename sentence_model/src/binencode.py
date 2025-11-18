import struct


def encode_int(value: int) -> bytes:
    """Encode an integer in bincode variable-length format."""
    if value < 251:
        return struct.pack("<B", value)
    elif value < 0xFFFF:
        return struct.pack("<BH", 251, value)
    elif value < 0xFFFFFFFF:
        return struct.pack("<BL", 252, value)
    else:
        return struct.pack("<BQ", 253, value)


def encode_qa_ranks_bincode(all_ranks: list[list[list[tuple[int, float]]]]) -> bytes:
    """
    Encode QA ranks to bincode format.

    Format: Vec<Vec<Vec<TrialQARank>>> where TrialQARank { answer_index: usize, score: f32 }
    Bincode serialization:
    - Vec length as u64 (little-endian)
    - Elements follow
    - usize is u64 on 64-bit systems
    - f32 is 4 bytes (little-endian)
    """
    buf = bytearray()

    # Outer Vec length (number of questions)
    buf.extend(encode_int(len(all_ranks)))

    for question_ranks in all_ranks:
        # Middle Vec length (number of keywords)
        buf.extend(encode_int(len(question_ranks)))

        for keyword_ranks in question_ranks:
            # Inner Vec length (number of ranks)
            buf.extend(encode_int(len(keyword_ranks)))

            for answer_index, score in keyword_ranks:
                # TrialQARank fields: answer_index (u32) + score (f32)
                buf.extend(encode_int(answer_index))
                buf.extend(struct.pack("<f", score))

    return bytes(buf)


def encode_aq_ranks_bincode(all_ranks: list[list[tuple[int, float]]]) -> bytes:
    """
    Encode AQ ranks to bincode format.

    Format: Vec<Vec<TrialQARank>>
    """
    buf = bytearray()

    # Outer Vec length
    buf.extend(encode_int(len(all_ranks)))

    for ranks in all_ranks:
        # Inner Vec length
        buf.extend(encode_int(len(ranks)))

        for answer_index, score in ranks:
            # TrialQARank fields
            buf.extend(encode_int(answer_index))
            buf.extend(struct.pack("<f", score))

    return bytes(buf)
