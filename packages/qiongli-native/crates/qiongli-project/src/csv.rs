// Bounded CSV reader shared by research artifacts and their Graph projection.
pub(crate) const MAX_TABLE_COLUMNS: usize = 16;
pub(crate) const MAX_TABLE_ROWS: usize = 2_048;
pub(crate) const MAX_FIELD_BYTES: usize = 8 * 1_024;

pub(crate) struct CsvRecord {
    pub(crate) line_number: usize,
    pub(crate) fields: Vec<String>,
}

pub(crate) fn parse_csv(text: &str) -> Result<Vec<CsvRecord>, ()> {
    let bytes = text.as_bytes();
    let mut records = Vec::new();
    let mut fields = Vec::new();
    let mut field = Vec::new();
    let mut index = 0usize;
    let mut line_number = 1usize;
    let mut record_line = 1usize;
    let mut in_quotes = false;
    let mut after_quote = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_quotes {
            if byte == b'"' {
                if bytes.get(index + 1) == Some(&b'"') {
                    field.push(b'"');
                    index += 1;
                } else {
                    in_quotes = false;
                    after_quote = true;
                }
            } else {
                if byte == b'\n' {
                    line_number += 1;
                }
                field.push(byte);
            }
        } else if after_quote {
            match byte {
                b',' => {
                    push_csv_field(&mut fields, &mut field)?;
                    after_quote = false;
                }
                b'\n' => {
                    push_csv_field(&mut fields, &mut field)?;
                    push_csv_record(&mut records, &mut fields, record_line)?;
                    line_number += 1;
                    record_line = line_number;
                    after_quote = false;
                }
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {}
                _ => return Err(()),
            }
        } else {
            match byte {
                b'"' if field.is_empty() => in_quotes = true,
                b'"' => return Err(()),
                b',' => push_csv_field(&mut fields, &mut field)?,
                b'\n' => {
                    push_csv_field(&mut fields, &mut field)?;
                    push_csv_record(&mut records, &mut fields, record_line)?;
                    line_number += 1;
                    record_line = line_number;
                }
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {}
                _ => field.push(byte),
            }
        }
        if field.len() > MAX_FIELD_BYTES {
            return Err(());
        }
        index += 1;
    }
    if in_quotes {
        return Err(());
    }
    if after_quote || !field.is_empty() || !fields.is_empty() {
        push_csv_field(&mut fields, &mut field)?;
        push_csv_record(&mut records, &mut fields, record_line)?;
    }
    Ok(records)
}

fn push_csv_field(fields: &mut Vec<String>, field: &mut Vec<u8>) -> Result<(), ()> {
    if fields.len() >= MAX_TABLE_COLUMNS || field.len() > MAX_FIELD_BYTES {
        return Err(());
    }
    let bytes = std::mem::take(field);
    fields.push(String::from_utf8(bytes).map_err(|_| ())?);
    Ok(())
}

fn push_csv_record(
    records: &mut Vec<CsvRecord>,
    fields: &mut Vec<String>,
    line_number: usize,
) -> Result<(), ()> {
    if records.len() > MAX_TABLE_ROWS {
        return Err(());
    }
    if fields.len() == 1 && fields[0].is_empty() {
        fields.clear();
        return Ok(());
    }
    records.push(CsvRecord {
        line_number,
        fields: std::mem::take(fields),
    });
    Ok(())
}
