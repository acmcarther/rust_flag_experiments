extern crate gconfig;
extern crate itertools;

use std::env::Args;
use itertools::Itertools;
use gconfig::ConfigMetadata;
use gconfig::ConfigInitializer;

pub struct FlagParser;

impl FlagParser {
  pub fn new() -> FlagParser {
    FlagParser
  }

  pub fn parse_from_args<I: Iterator<Item = String>>(&self, args: I) {
    let initializers = gconfig::STATIC_CONFIG_INITIALIZERS.read()
      .expect("initializers were poisoned");

    let mut initializer_meta_sorted =
      initializers.iter()
        .map(|i| i.metadata().clone())
        .collect::<Vec<ConfigMetadata>>();
    initializer_meta_sorted.sort_by(|a, b| a.config_name().cmp(b.config_name()));

    let flag_name_conflicts: Vec<(String, Vec<ConfigMetadata>)> =
      initializer_meta_sorted.into_iter()
        .group_by(|i| i.config_name().to_owned())
        .into_iter()
        .map(|(key, metadata_objects)| (key, metadata_objects.collect::<Vec<ConfigMetadata>>()))
        .filter(|&(_, ref metadata_objects)| metadata_objects.len() > 1)
        .collect();

    // TODO: Something more user friendly
    assert_eq!(flag_name_conflicts, Vec::new());

    let arg_elements = args.map(|content| {
      if content == "--".to_owned() {
        ArgComponent::Terminator
      } else if content.starts_with("--") {
        // TODO: This isn't perfect -- more robust parsing later
        if content.contains('=') {
          ArgComponent::CompleteArg(content)
        } else {
          ArgComponent::ArgPrefix(content)
        }
      } else {
        ArgComponent::ArgSuffix(content)
      }
    })
    .peekable()
    .batching(|mut it| {
      let first = it.next();
      if first.is_none() { return None }
      match first {
        None => None,
        Some(ArgComponent::Terminator) => None,
        Some(ArgComponent::ArgSuffix(value)) => {
          Some(Err(format!("Arg element [{}] did not have a corresponding key", value)))
        },
        Some(ArgComponent::CompleteArg(name_and_value)) =>  {
          let eq_byte_idx = name_and_value.find('=').unwrap();
          let (name, value_plus_eq) = name_and_value.split_at(eq_byte_idx);
          Some(Ok(ArgCapture {
            label: name.chars().skip(2 /* -- */).collect::<String>(),
            value: Some(value_plus_eq.chars().skip(1 /* = */).collect::<String>()),
          }))
        }
        Some(ArgComponent::ArgPrefix(name)) => {
          let mut value_opt = None;
          if let Some(&ArgComponent::ArgSuffix(ref s)) = it.peek() {
            // TODO: Fix janky clone
            value_opt = Some(s.clone())
          }
          match value_opt {
            Some(value) => {
              // Toss next element
              it.next();
              Some(Ok(ArgCapture {
                label: name.chars().skip(2 /* == */).collect::<String>(),
                value: Some(value.to_owned()),
              }))
            },
            None => {
              Some(Ok(ArgCapture {
                label: name.chars().skip(2 /* == */).collect::<String>(),
                value: None,
              }))
            }
          }
        }
      }
    })
    .collect::<Vec<Result<ArgCapture, String>>>();


    let mut captures = Vec::new();
    for arg in arg_elements.into_iter() {
      if let Err(e) = arg {
        panic!(e)
      }

      captures.push(arg.unwrap());
    }

    println!("{:?}", captures);
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArgComponent {
  CompleteArg(String),
  ArgPrefix(String),
  ArgSuffix(String),
  Terminator,
}

#[derive(Clone,Debug, PartialEq, Eq)]
struct ArgCapture {
  label: String,
  value: Option<String>
}

fn main() {
  use std::env;

  FlagParser::new().parse_from_args(env::args().skip(1))
}

#[cfg(test)]
mod tests {
  use std::env;

  #[test]
  fn it_works() {
    panic!(format!("{:?}", env::args().collect::<Vec<_>>()));
  }
}
