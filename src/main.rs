use std::fs;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
// rcli csv -i input.csv -o output.json --header -d ','
use rcli::{
    get_content, get_reader, process_csv, process_decode, process_encode, process_genpass,
    process_text_decrypt, process_text_encrypt, process_text_key_generate, process_text_sign,
    process_text_verify,
};
use rcli::{Base64SubCommand, Opts, SubCommand, TextSubCommand};

use clap::Parser;
use zxcvbn::zxcvbn;

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    match opts.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, output, opts.format)?;
        }
        SubCommand::GenPass(opts) => {
            let ret = process_genpass(
                opts.length,
                opts.uppercase,
                opts.lowercase,
                opts.number,
                opts.symbol,
            )?;
            println!("{}", ret);

            let estimate = zxcvbn(&ret, &[])?;
            eprintln!("Estimated strength: {}", estimate.score());
        }
        SubCommand::Base64(subcmd) => match subcmd {
            Base64SubCommand::Encode(opts) => {
                let mut reader = get_reader(&opts.input)?;
                let ret = process_encode(&mut reader, opts.format)?;
                println!("{}", ret);
            }
            Base64SubCommand::Decode(opts) => {
                let mut reader = get_reader(&opts.input)?;
                let ret = process_decode(&mut reader, opts.format)?;
                println!("{}", ret);
            }
        },

        SubCommand::Text(subtext) => match subtext {
            TextSubCommand::Sign(opts) => {
                let mut reader = get_reader(&opts.input)?;
                let key = get_content(&opts.key)?;
                let sig = process_text_sign(&mut reader, &key, opts.format)?;
                let encoded = URL_SAFE_NO_PAD.encode(sig);
                println!("{}", encoded);
            }
            TextSubCommand::Verify(opts) => {
                let mut reader = get_reader(&opts.input)?;
                let key = get_content(&opts.key)?;
                let decoded = URL_SAFE_NO_PAD.decode(&opts.sig)?;
                println!("decode len: {}", decoded.len());
                let verified = process_text_verify(&mut reader, &key, &decoded, opts.format)?;
                if verified {
                    println!("✓ Signature verified");
                } else {
                    println!("⚠ Signature not verified");
                }
            }
            TextSubCommand::Generate(opts) => {
                let key = process_text_key_generate(opts.format)?;
                for (k, v) in key {
                    fs::write(opts.output_path.join(k), v)?;
                }
            }
            TextSubCommand::Encrypt(opts) => {
                let mut reader = get_reader(&opts.input)?;
                let key = get_content(&opts.key)?;
                let sig = process_text_encrypt(&mut reader, &key, opts.format)?;
                let encoded = URL_SAFE_NO_PAD.encode(sig);
                println!("{}", encoded);
            }
            TextSubCommand::Decrypt(opts) => {
                let mut reader = get_reader(&opts.input)?;
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                let b64_encrypt_text = String::from_utf8(buf)?;
                let encrypt_text = URL_SAFE_NO_PAD.decode(b64_encrypt_text)?;

                let key = get_content(&opts.key)?;
                let plaintext = process_text_decrypt(&encrypt_text[..], &key, opts.format)?;
                println!("decrypt res: {}", String::from_utf8(plaintext)?);
            }
        },
    }

    Ok(())
}
