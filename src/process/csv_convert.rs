use crate::opt::OutputFormat;
use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use simple_excel_writer::*;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Player {
    name: String,
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit: u8,
}

pub fn process_csv(input: &str, output: String, format: OutputFormat) -> anyhow::Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::with_capacity(128);
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        let record = result?;
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();
        ret.push(json_value);
    }

    match format {
        OutputFormat::Json => {
            let content = serde_json::to_string_pretty(&ret)?;
            fs::write(output, content)?;
        }
        OutputFormat::Yaml => {
            let content = serde_yaml::to_string(&ret)?;
            fs::write(output, content)?;
        }
        OutputFormat::Xlsx => {
            let mut wb = Workbook::create(&output);
            let mut sheet = wb.create_sheet("Sheet1");

            // set column width
            sheet.add_column(Column { width: 20.0 });
            sheet.add_column(Column { width: 10.0 });
            sheet.add_column(Column { width: 20.0 });
            sheet.add_column(Column { width: 20.0 });
            sheet.add_column(Column { width: 30.0 });

            wb.write_sheet(&mut sheet, |sheet_writer| {
                for rows in ret.as_slice() {
                    let mut row = Row::new();
                    for (_, val) in rows.as_object().unwrap().iter() {
                        row.add_cell(val.to_string().trim_matches('"'));
                    }
                    sheet_writer.append_row(row)?;
                }
                Ok(())
            })?;

            wb.close()?;
        }
    };

    Ok(())
}
