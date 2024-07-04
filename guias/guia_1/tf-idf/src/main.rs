use std::collections::{HashMap, HashSet};
use std::fs::{read_dir, read_to_string, File};
use std::io::{BufRead, BufReader, Result};

#[allow(dead_code)]
struct TFIDFModel {
    words: HashMap<String, Vec<String>>,
    corpus_size: usize,
    docs: HashMap<String, f64>,
}

/*
1. Tokenizar un texto
    - Leer el texto
    - Pasar a minúsculas
    - Quitar signos de puntuación
    - Separar por espacios
    - Quitar stopwords
    - Stemming
    - Retornar un vector de palabras
    tip: Usar un HashSet<String> para las stopwords

2. Leer el corpus y llenar la tabla donde:
    k -> palabra
    v -> lista con los ids del texto donde aparece
    tip: Usar un HashMap<String, Vec<usize>>

3. Recibir la Query del usuario donde:
    - Leer la query
    - Pasar a minúsculas
    - Quitar signos de puntuación
    - Separar por espacios
    - Quitar stopwords
    - Stemming
    - Retornar un vector de palabras

4. Calcular tf-idf de los documentos

*/

fn get_stopwords(path: &str) -> Result<HashSet<String>> {
    let mut stopwords = HashSet::<String>::new();
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader.lines().for_each(|line| {
        if let Ok(line) = line {
            stopwords.insert(line.to_lowercase());
        }
    });

    Ok(stopwords)
}

fn token_file(path: &str, stopwords: &HashSet<String>) -> Result<Vec<String>> {
    let str = match read_to_string(path) {
        Ok(str) => str,
        Err(_) => "".to_string(),
    };

    let words = str
        .split_whitespace()
        .map(|word| word.replace(|c: char| !c.is_alphabetic(), ""))
        .map(|word| word.to_lowercase())
        .filter(|word| !stopwords.contains(word) && word.len() > 0)
        .collect::<Vec<String>>();
    Ok(words)
}

fn generate_indices(path: &str, stopwords: &HashSet<String>) -> Result<TFIDFModel> {
    let mut indices = HashMap::<String, Vec<String>>::new();
    let mut docs = HashMap::<String, f64>::new();
    let dir = read_dir(path)?;
    let mut size = 0_usize;

    dir.into_iter().for_each(|entry| {
        if let Ok(entry) = entry {
            let path = entry.path();
            let name = entry.file_name().to_str().unwrap().to_string();
            docs.insert(name.clone(), 0_f64);

            let words = token_file(path.to_str().unwrap(), &stopwords).unwrap();
            for word in words {
                if let Some(value) = indices.get_mut(&word) {
                    value.push(name.clone());
                } else {
                    indices.insert(word, vec![name.clone()]);
                }
            }
            size += 1;
        }
    });

    Ok(TFIDFModel {
        words: indices,
        corpus_size: size,
        docs,
    })
}

fn get_docs(model: &TFIDFModel, query: Vec<String>) -> Vec<String> {
    let mut query_docs = HashSet::<String>::new();
    for word in query {
        if let Some(docs) = model.words.get(&word) {
            for doc in docs {
                query_docs.insert(doc.clone());
            }
        }
    }

    query_docs.into_iter().collect::<Vec<String>>()
}

fn tf_idf(model: &TFIDFModel, docs: &String) {}

#[allow(dead_code)]
fn docs_tfidf(mut model: TFIDFModel, docs: &Vec<String>) -> Vec<(String, f64)> {
    for doc in docs {
        if let Some(value) = model.docs.get_mut(doc) {
            *value += tf_idf(&model, doc);
        }
    }

    println!("{:?}", model.docs);
    Vec::new()
}

#[allow(unused_variables)]
fn main() -> Result<()> {
    // std::env::set_var("RUST_BACKTRACE", "1");
    let stopwords = get_stopwords("src/stopwords.txt")?;
    let model = generate_indices("src/corpus", &stopwords)?;

    let query = &std::env::args()
        .skip(1)
        .map(|x| x.replace(|c: char| !c.is_alphabetic(), ""))
        .map(|x| x.to_lowercase())
        .filter(|x| !stopwords.contains(x))
        .collect::<Vec<String>>();

    let docs = get_docs(&model, query.to_vec());
    let arr = docs_tfidf(model, &docs);

    // println!("{:?}", arr);

    Ok(())
}
