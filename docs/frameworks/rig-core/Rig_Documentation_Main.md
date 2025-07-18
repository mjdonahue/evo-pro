Rig is a Rust library for building LLM-powered applications that focuses on ergonomics and modularity.

## Table of contents

- [High-level features](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/index.html#high-level-features)
- [Simple Example](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/index.html#simple-example)
- [Core Concepts](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/index.html#core-concepts)
- [Integrations](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/index.html#integrations)

## High-level features

- Full support for LLM completion and embedding workflows
- Simple but powerful common abstractions over LLM providers (e.g. OpenAI, Cohere) and vector stores (e.g. MongoDB, in-memory)
- Integrate LLMs in your app with minimal boilerplate

## Simple example:

```
use rig::{completion::Prompt, providers::openai};

#[tokio::main]
async fn main() {
    // Create OpenAI client and agent.
    // This requires the `OPENAI_API_KEY` environment variable to be set.
    let openai_client = openai::Client::from_env();

    let gpt4 = openai_client.agent("gpt-4").build();

    // Prompt the model and print its response
    let response = gpt4
        .prompt("Who are you?")
        .await
        .expect("Failed to prompt GPT-4");

    println!("GPT-4: {response}");
}
```

Note: using `#[tokio::main]` requires you enable tokio’s `macros` and `rt-multi-thread` features or just `full` to enable all features (`cargo add tokio --features macros,rt-multi-thread`).

## Core concepts

### Completion and embedding models

Rig provides a consistent API for working with LLMs and embeddings. Specifically, each provider (e.g. OpenAI, Cohere) has a `Client` struct that can be used to initialize completion and embedding models. These models implement the [CompletionModel](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/completion/request/trait.CompletionModel.html "trait rig::completion::request::CompletionModel") and [EmbeddingModel](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/embeddings/embedding/trait.EmbeddingModel.html "trait rig::embeddings::embedding::EmbeddingModel") traits respectively, which provide a common, low-level interface for creating completion and embedding requests and executing them.

### Agents

Rig also provides high-level abstractions over LLMs in the form of the [Agent](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/agent/struct.Agent.html "struct rig::agent::Agent") type.

The [Agent](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/agent/struct.Agent.html "struct rig::agent::Agent") type can be used to create anything from simple agents that use vanilla models to full blown RAG systems that can be used to answer questions using a knowledge base.

### Vector stores and indexes

Rig provides a common interface for working with vector stores and indexes. Specifically, the library provides the [VectorStoreIndex](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/vector_store/trait.VectorStoreIndex.html "trait rig::vector_store::VectorStoreIndex") trait, which can be implemented to define vector stores and indices respectively. Those can then be used as the knowledge base for a RAG enabled [Agent](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/agent/struct.Agent.html "struct rig::agent::Agent"), or as a source of context documents in a custom architecture that use multiple LLMs or agents.

## Integrations

### Model Providers

Rig natively supports the following completion and embedding model provider integrations:

- OpenAI
- Cohere
- Anthropic
- Perplexity
- Google Gemini
- xAI
- DeepSeek

You can also implement your own model provider integration by defining types that implement the [CompletionModel](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/completion/request/trait.CompletionModel.html "trait rig::completion::request::CompletionModel") and [EmbeddingModel](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/embeddings/embedding/trait.EmbeddingModel.html "trait rig::embeddings::embedding::EmbeddingModel") traits.

### Vector Stores

Rig currently supports the following vector store integrations via companion crates:

- `rig-mongodb`: Vector store implementation for MongoDB
- `rig-lancedb`: Vector store implementation for LanceDB
- `rig-neo4j`: Vector store implementation for Neo4j
- `rig-qdrant`: Vector store implementation for Qdrant

You can also implement your own vector store integration by defining types that implement the [VectorStoreIndex](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/vector_store/trait.VectorStoreIndex.html "trait rig::vector_store::VectorStoreIndex") trait.

## Re-exports

`pub use completion::[message](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/completion/message/index.html "mod rig::completion::message");`

`pub use embeddings::[Embed](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/embeddings/embed/trait.Embed.html "trait rig::embeddings::embed::Embed");`

`pub use one_or_many::[EmptyListError](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/one_or_many/struct.EmptyListError.html "struct rig::one_or_many::EmptyListError");`

`pub use one_or_many::[OneOrMany](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/one_or_many/struct.OneOrMany.html "struct rig::one_or_many::OneOrMany");`

## Modules

[agent](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/agent/index.html "mod rig::agent")

This module contains the implementation of the [Agent](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/agent/struct.Agent.html "struct rig::agent::Agent") struct and its builder.

[cli_chatbot](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/cli_chatbot/index.html "mod rig::cli_chatbot")

[client](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/client/index.html "mod rig::client")

This module provides traits for defining and creating provider clients. Clients are used to create models for completion, embeddings, etc. Dyn-compatible traits have been provided to allow for more provider-agnostic code.

[completion](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/completion/index.html "mod rig::completion")

[embeddings](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/embeddings/index.html "mod rig::embeddings")

This module provides functionality for working with embeddings. Embeddings are numerical representations of documents or other objects, typically used in natural language processing (NLP) tasks such as text classification, information retrieval, and document similarity.

[extractor](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/extractor/index.html "mod rig::extractor")

This module provides high-level abstractions for extracting structured data from text using LLMs.

[loaders](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/loaders/index.html "mod rig::loaders")

This module provides utility structs for loading and preprocessing files.

[one_or_many](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/one_or_many/index.html "mod rig::one_or_many")

[pipeline](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/pipeline/index.html "mod rig::pipeline")

This module defines a flexible pipeline API for defining a sequence of operations that may or may not use AI components (e.g.: semantic search, LLMs prompting, etc).

[prelude](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/prelude/index.html "mod rig::prelude")

[providers](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/providers/index.html "mod rig::providers")

This module contains clients for the different LLM providers that Rig supports.

[streaming](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/streaming/index.html "mod rig::streaming")

This module provides functionality for working with streaming completion models. It provides traits and types for generating streaming completion requests and handling streaming completion responses.

[tool](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/tool/index.html "mod rig::tool")

Module defining tool related structs and traits.

[transcription](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/transcription/index.html "mod rig::transcription")

This module provides functionality for working with audio transcription models. It provides traits, structs, and enums for generating audio transcription requests, handling transcription responses, and defining transcription models.

[vector_store](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/vector_store/index.html "mod rig::vector_store")

## Macros

[conditional](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.conditional.html "macro rig::conditional")

Creates an `Op` that conditionally dispatches to one of multiple sub-ops based on the variant of the input enum.

[impl_audio_generation](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.impl_audio_generation.html "macro rig::impl_audio_generation")

[impl_conversion_traits](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.impl_conversion_traits.html "macro rig::impl_conversion_traits")

Implements the conversion traits for a given struct

[impl_image_generation](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.impl_image_generation.html "macro rig::impl_image_generation")

[parallel](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.parallel.html "macro rig::parallel")

[parallel_internal](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.parallel_internal.html "macro rig::parallel_internal")

[parallel_op](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.parallel_op.html "macro rig::parallel_op")

[try_conditional](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.try_conditional.html "macro rig::try_conditional")

Creates a `TryOp` that conditionally dispatches to one of multiple sub-ops based on the variant of the input enum, returning a `Result`.

[try_parallel](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.try_parallel.html "macro rig::try_parallel")

[try_parallel_internal](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.try_parallel_internal.html "macro rig::try_parallel_internal")

[tuple_pattern](https://docs.rs/rig-core/latest/x86_64-apple-darwin/rig/macro.tuple_pattern.html "macro rig::tuple_pattern")