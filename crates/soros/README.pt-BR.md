# soros

[![crates.io](https://img.shields.io/crates/v/soros.svg)](https://crates.io/crates/soros)
[![docs.rs](https://img.shields.io/docsrs/soros)](https://docs.rs/soros)
[![Licença: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Um interpretador da linguagem de programação **Soros**: a pequena DSL de
reescrita por regex que o projeto
[Numbertext.org](https://numbertext.github.io/) usa para descrever a
conversão de número para texto, um arquivo de regras por idioma.

Read this in [English](README.md).

```rust
// O programa de inversão de string da própria especificação.
let program = soros::Program::compile(r"(.*)(.) \2$1", "");

assert_eq!(program.run("hello"), "olleh");
```

## Escopo

Esta crate implementa a linguagem e nada mais. Ela não tem noção de
números, cardinais, moedas ou idiomas — tudo que é específico da
numbertext vive na crate
[`numbertext`](https://crates.io/crates/numbertext), que é o motivo usual
para se chegar até esta aqui.

Manter as duas separadas é justamente o ponto, e é por isso que esta é uma
crate separada em vez de um módulo privado: a `soros` não tem dependência
de `numbertext`, então a fronteira é imposta pelo Cargo, e não por uma
convenção que alguém precisa lembrar.

## A linguagem, condensada

A especificação completa é o `doc/sorosspec.pdf` no
[repositório upstream](https://github.com/Numbertext/libnumbertext).

- Um programa é uma lista única, plana e ordenada de regras
  `padrão → template`, sempre aplicada como **casamento completo** contra
  a string de entrada atual. O primeiro casamento vence; nenhum casamento
  produz uma string vazia.
- `== prefixo ==` define uma macro: o prefixo é concatenado ao padrão de
  cada regra seguinte, até o próximo `==`.
- Sintaxe de template: `\N` (backreference bruta), `$N` (açúcar para
  `$(\N)`), `$(...)` (chamada recursiva sobre um argumento construído),
  `[...]` (texto condicional, mantido apenas se as chamadas dentro dele
  forem não vazias) e `|` (modificador de fronteira).
- Uma regra oculta está sempre ativa, removendo zeros à esquerda
  excedentes.

## Motor de regex

Os padrões compilam via [`fancy-regex`](https://docs.rs/fancy-regex), que
delega ao motor de tempo linear [`regex`](https://docs.rs/regex) qualquer
padrão sem backreference ou lookaround, e usa o motor de backtracking
próprio apenas para o restante — do qual arquivos de regras reais
genuinamente precisam. O backtracking é limitado por `BACKTRACK_LIMIT`,
uma contagem de passos e não um tempo de relógio, de modo que o
comportamento é reprodutível entre máquinas. A avaliação é limitada por
`MAX_RECURSION_DEPTH`.

## Robustez

`Program::compile` é infalível. Uma regra que ele não consegue compilar
fica registrada em `Program::skipped` em vez de derrubar o programa
inteiro, então uma linha malformada nunca leva junto um arquivo de regras
— e é reportada, nunca adivinhada.

## Licença

MIT, veja [LICENSE](LICENSE). Note que os **arquivos de regras** `.sor`
distribuídos pela crate `numbertext` são outra história: eles vêm do
libnumbertext upstream sob BSD-3-Clause. Esta crate não distribui nenhum
arquivo de regras.
