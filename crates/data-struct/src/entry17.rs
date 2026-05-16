use crate::{checked_slice, inflate_linkdata_entry, Result};

pub const LINKDATA_ENTRY_INDEX: usize = 17;
pub const CANDIDATE_RECORD_ID_OFFSET: usize = 0x20;
pub const CANDIDATE_RECORD_SIZE: usize = 0x44;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry17 {
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry17Occurrence {
    pub offset: usize,
    pub value: u32,
    pub width: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry17Window {
    pub start: usize,
    pub end: usize,
    pub raw: Vec<u8>,
    pub hits: Vec<Entry17Occurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry17RecordCandidate {
    pub id: u32,
    pub offset: usize,
    pub id_offset: usize,
    pub raw: Vec<u8>,
    pub fields: Vec<u32>,
}

impl Entry17 {
    pub fn parse(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    pub fn from_linkdata_bytes(linkdata: &[u8]) -> Result<Self> {
        let bytes = inflate_linkdata_entry(linkdata, LINKDATA_ENTRY_INDEX)?;
        Ok(Self::parse(bytes))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn occurrences_for_value(&self, value: u32) -> Vec<Entry17Occurrence> {
        let mut occurrences = Vec::new();

        if value <= u16::MAX as u32 {
            let pattern = (value as u16).to_le_bytes();
            occurrences.extend(
                pattern_offsets(&self.bytes, &pattern)
                    .into_iter()
                    .map(|offset| Entry17Occurrence {
                        offset,
                        value,
                        width: 2,
                    }),
            );
        }

        let pattern = value.to_le_bytes();
        occurrences.extend(
            pattern_offsets(&self.bytes, &pattern)
                .into_iter()
                .map(|offset| Entry17Occurrence {
                    offset,
                    value,
                    width: 4,
                }),
        );

        occurrences
    }

    pub fn occurrences_for_values(&self, values: &[u32]) -> Vec<Entry17Occurrence> {
        let mut occurrences = values
            .iter()
            .flat_map(|value| self.occurrences_for_value(*value))
            .collect::<Vec<_>>();
        occurrences.sort_by_key(|hit| (hit.offset, hit.width, hit.value));
        occurrences
    }

    pub fn context(&self, offset: usize, radius: usize) -> Result<Entry17Window> {
        let start = offset.saturating_sub(radius);
        let end = self.bytes.len().min(offset.saturating_add(radius));
        let raw = checked_slice(&self.bytes, start, end - start, "entry17 context")?.to_vec();
        Ok(Entry17Window {
            start,
            end,
            raw,
            hits: Vec::new(),
        })
    }

    pub fn clustered_windows(
        &self,
        occurrences: &[Entry17Occurrence],
        radius: usize,
        merge_gap: usize,
    ) -> Vec<Entry17Window> {
        let mut hits = occurrences.to_vec();
        hits.sort_by_key(|hit| (hit.offset, hit.width, hit.value));

        let mut windows = Vec::<Entry17Window>::new();
        for hit in hits {
            let hit_end = hit.offset.saturating_add(hit.width);
            let start = hit.offset.saturating_sub(radius);
            let end = self.bytes.len().min(hit_end.saturating_add(radius));

            if let Some(last) = windows.last_mut() {
                let last_hit_end = last
                    .hits
                    .last()
                    .map(|last_hit| last_hit.offset.saturating_add(last_hit.width))
                    .unwrap_or(last.end);
                if hit.offset <= last_hit_end.saturating_add(merge_gap) {
                    last.end = last.end.max(end);
                    last.raw = self.bytes[last.start..last.end].to_vec();
                    last.hits.push(hit);
                    continue;
                }
            }

            windows.push(Entry17Window {
                start,
                end,
                raw: self.bytes[start..end].to_vec(),
                hits: vec![hit],
            });
        }
        windows
    }

    pub fn record_candidates_for_id(&self, id: u32) -> Result<Vec<Entry17RecordCandidate>> {
        self.occurrences_for_value(id)
            .into_iter()
            .filter(|hit| hit.width == 4)
            .map(|hit| self.record_candidate_from_id_offset(id, hit.offset))
            .collect()
    }

    pub fn record_candidates_for_ids(&self, ids: &[u32]) -> Result<Vec<Entry17RecordCandidate>> {
        let mut records = Vec::new();
        for id in ids {
            records.extend(self.record_candidates_for_id(*id)?);
        }
        records.sort_by_key(|record| (record.offset, record.id));
        Ok(records)
    }

    pub fn clone_record_candidate(&mut self, source_id: u32, target_id: u32) -> Result<bool> {
        let source_records = self.record_candidates_for_id(source_id)?;
        let target_records = self.record_candidates_for_id(target_id)?;
        let (Some(source), Some(target)) = (source_records.first(), target_records.first()) else {
            return Ok(false);
        };
        if source_records.len() != 1 || target_records.len() != 1 {
            return Ok(false);
        }

        let mut raw = source.raw.clone();
        raw[CANDIDATE_RECORD_ID_OFFSET..CANDIDATE_RECORD_ID_OFFSET + 4]
            .copy_from_slice(&target_id.to_le_bytes());
        self.bytes[target.offset..target.offset + raw.len()].copy_from_slice(&raw);
        Ok(true)
    }

    fn record_candidate_from_id_offset(
        &self,
        id: u32,
        id_offset: usize,
    ) -> Result<Entry17RecordCandidate> {
        let offset = id_offset.saturating_sub(CANDIDATE_RECORD_ID_OFFSET);
        let end = self.bytes.len().min(offset + CANDIDATE_RECORD_SIZE);
        let raw = checked_slice(
            &self.bytes,
            offset,
            end - offset,
            "entry17 record candidate",
        )?
        .to_vec();
        let fields = raw
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect::<Vec<_>>();
        Ok(Entry17RecordCandidate {
            id,
            offset,
            id_offset,
            raw,
            fields,
        })
    }
}

fn pattern_offsets(bytes: &[u8], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() || bytes.len() < pattern.len() {
        return Vec::new();
    }
    bytes
        .windows(pattern.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == pattern).then_some(offset))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_u16_and_u32_occurrences_for_watched_values() {
        let entry = Entry17::parse([
            0x1a, 0x00, 0x00, 0x00, // 26 as u32 and u16 at offset 0
            0xbb, 0x02, // 699 as u16 at offset 4
            0x8e, 0x03, 0x00, 0x00, // 910 noise, not watched here
            0x8f, 0x03, 0x00, 0x00, // 911 as u32 and u16 at offset 10
        ]);

        let occurrences = entry.occurrences_for_values(&[26, 699, 911]);

        assert!(occurrences.contains(&Entry17Occurrence {
            offset: 0,
            value: 26,
            width: 2,
        }));
        assert!(occurrences.contains(&Entry17Occurrence {
            offset: 0,
            value: 26,
            width: 4,
        }));
        assert!(occurrences.contains(&Entry17Occurrence {
            offset: 4,
            value: 699,
            width: 2,
        }));
        assert!(occurrences.contains(&Entry17Occurrence {
            offset: 10,
            value: 911,
            width: 4,
        }));
    }

    #[test]
    fn builds_context_windows_around_nearby_occurrences() {
        let bytes = (0u8..32).collect::<Vec<_>>();
        let entry = Entry17::parse(bytes);
        let occurrences = vec![
            Entry17Occurrence {
                offset: 8,
                value: 26,
                width: 2,
            },
            Entry17Occurrence {
                offset: 12,
                value: 699,
                width: 2,
            },
            Entry17Occurrence {
                offset: 28,
                value: 911,
                width: 2,
            },
        ];

        let windows = entry.clustered_windows(&occurrences, 4, 8);

        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].start, 4);
        assert_eq!(windows[0].end, 18);
        assert_eq!(windows[0].hits.len(), 2);
        assert_eq!(windows[1].start, 24);
        assert_eq!(windows[1].end, 32);
        assert_eq!(windows[1].hits.len(), 1);
    }

    #[test]
    fn extracts_record_candidate_with_id_at_offset_0x20() {
        let mut bytes = vec![0xff; CANDIDATE_RECORD_SIZE + 0x10];
        let record_offset = 0x10;
        bytes[record_offset + 0x00..record_offset + 0x04]
            .copy_from_slice(&0x11223344u32.to_le_bytes());
        bytes[record_offset + CANDIDATE_RECORD_ID_OFFSET
            ..record_offset + CANDIDATE_RECORD_ID_OFFSET + 4]
            .copy_from_slice(&699u32.to_le_bytes());
        bytes[record_offset + 0x24..record_offset + 0x28].copy_from_slice(&6u32.to_le_bytes());

        let entry = Entry17::parse(bytes);
        let records = entry.record_candidates_for_id(699).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].offset, record_offset);
        assert_eq!(
            records[0].id_offset,
            record_offset + CANDIDATE_RECORD_ID_OFFSET
        );
        assert_eq!(records[0].fields[0], 0x11223344);
        assert_eq!(records[0].fields[8], 699);
        assert_eq!(records[0].fields[9], 6);
    }

    #[test]
    fn clones_source_candidate_over_target_and_rewrites_only_id_field() {
        let mut bytes = vec![0xff; 0x100];
        put_record(
            &mut bytes,
            0x10,
            &[
                30,
                u32::MAX,
                u32::MAX,
                27,
                5,
                65793,
                4294967042,
                769,
                586,
                5,
                25,
            ],
        );
        put_record(
            &mut bytes,
            0x60,
            &[
                u32::MAX,
                u32::MAX,
                u32::MAX,
                0,
                0,
                255,
                u32::MAX,
                256,
                699,
                6,
                36,
            ],
        );
        let mut entry = Entry17::parse(bytes);

        let patched = entry.clone_record_candidate(586, 699).unwrap();

        assert!(patched);
        let target = entry.record_candidates_for_id(699).unwrap().remove(0);
        assert_eq!(target.offset, 0x60);
        assert_eq!(target.fields[0], 30);
        assert_eq!(target.fields[3], 27);
        assert_eq!(target.fields[4], 5);
        assert_eq!(target.fields[8], 699);
        assert_eq!(target.fields[9], 5);
        assert_eq!(target.fields[10], 25);
    }

    fn put_record(bytes: &mut [u8], offset: usize, fields: &[u32]) {
        for (index, field) in fields.iter().enumerate() {
            let field_offset = offset + index * 4;
            bytes[field_offset..field_offset + 4].copy_from_slice(&field.to_le_bytes());
        }
    }
}
