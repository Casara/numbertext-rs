# Guia de Estilo Rust

*[Read in English](coding-style.md)*

## Propósito

Este documento registra as convenções que o projeto adota **além** do que
o `rustfmt` e o `clippy` já impõem automaticamente. Se uma ferramenta
consegue verificar, não está escrito aqui — está no `rustfmt.toml`, no
`clippy.toml` ou na tabela de lints do workspace no `Cargo.toml`.

As decisões priorizam, nesta ordem:

* legibilidade;
* consistência;
* manutenibilidade;
* qualidade do diff.

---

## Formatação

Todo o código é formatado com o `rustfmt`, nos padrões dele. O
`rustfmt.toml` define apenas a edição.

Isso é deliberado: os padrões do `rustfmt` são o que qualquer leitor de
Rust já espera, e um estilo próprio custa mais em estranhamento do que
rende em preferência. Não formate à mão contra a ferramenta.

---

## Lints

A tabela de lints do workspace no `Cargo.toml` liga `clippy::pedantic` e
`clippy::cargo` para todas as crates, e o CI roda o clippy com
`-D warnings`.

Quando um lint pedantic estiver errado para um caso específico, prefira
corrigir o código a silenciar o lint. `render_node` recebendo quatro
booleanos soltos era um problema real de projeto que o
`clippy::fn_params_excessive_bools` apontou corretamente; a correção foi
o tipo `Boundary`, não um `#[allow]`.

Onde um `#[allow]` for genuinamente a resposta certa, dê a ele o escopo
mais estreito possível e diga o porquê num comentário na mesma linha ou
logo acima. Um `#[allow]` sem justificativa é um TODO ao qual ninguém
voltará.

---

## Legibilidade acima de concisão

Prefira código explícito e fácil de ler a código maximamente compacto.

Preferido:

```rust
let Some(caps) = rule.re.captures(input) else {
    continue;
};
```

Evite encadeamentos que empacotam três decisões numa única expressão
quando elas se leriam como três passos.

---

## Comentários

Comentários devem explicar:

* propósito;
* comportamento;
* limitações;
* **decisões de projeto, especialmente as alternativas que foram
  rejeitadas.**

Esse último item carrega a maior parte do peso neste repositório. O
conhecimento caro aqui não é o que o código faz — é por que ele não é
feito da outra maneira, aparentemente mais simples. Um leitor que não
encontrar isso registrado vai, mais cedo ou mais tarde, "simplificar" de
volta.

Comentários que apenas repetem o nome do item devem ser evitados.

Ruim:

```rust
/// Compila um programa.
pub fn compile(source: &str, lang: &str) -> Self
```

Melhor:

```rust
/// Compila código-fonte Soros num programa executável para o código de
/// idioma informado.
///
/// É infalível por projeto. Não existe fonte que falhe ao ser tokenizada
/// - toda falha é por regra e fica registrada em [`Program::skipped`],
/// de modo que uma linha malformada nunca derruba um locale inteiro.
```

---

## Comentários de documentação são testes

O exemplo de todo item público é um doctest, executado por
`cargo test --doc`. Os dois READMEs também estão ligados como doctests
(veja `ReadmeExamples` em `src/lib.rs`), então os exemplos deles são
compilados e executados, e não apenas plausíveis.

Escreva exemplos que afirmam, não exemplos que imprimem:

```rust
/// ```
/// # let en = numbertext::Locale::new("en").unwrap();
/// assert_eq!(en.ordinal("123"), "one hundred twenty-third");
/// ```
```

Um exemplo com `assert_eq!` é uma especificação que o compilador
verifica. Um exemplo com `println!` só prova que o código compila.

**Execute o código para obter o valor esperado.** A saída dos locales
codifica escolhas estilísticas reais de cada idioma, fáceis de errar por
intuição — veja "Verify locale output before writing it down" no
AGENTS.md.

---

## Erros

O tipo de erro público é um enum `#[non_exhaustive]` com uma variante por
falha distinguível, casado pelos chamadores em vez de analisado a partir
da mensagem.

### Modele o que de fato pode falhar

A decisão mais importante de tratamento de erros nesta crate é o que
**não** está em `Error`. A conversão não pode falhar: a linguagem Soros
não tem noção de conversão malsucedida, então uma entrada que nenhuma
regra casa produz uma string vazia. Os métodos de conversão de `Locale`
devolvem `String`, não `Result<String, _>`.

Resista à tentação de tornar tudo falível por simetria. Um `Result` que o
chamador só pode `.unwrap()` é ruído que ensina as pessoas a ignorar os
que importam.

### Mensagens

Minúsculas, sem ponto final, descrevendo o que não pôde ser feito — e
nomeando a coisa com a qual não pôde ser feito.

Correto:

```rust
write!(f, "unknown language {lang:?} (see `languages()` for the supported base codes)")
```

Evite invólucros genéricos como "error", "failed" ou "unexpected error",
e evite repetir contexto que o chamador já tem.

### O texto da mensagem não é API

Chamadores casam sobre variantes. É por isso que a redação pode carregar
uma frase inteira de orientação sem virar um risco de compatibilidade.

---

## Panics

Código de biblioteca não entra em pânico por causa da entrada do
chamador. `expect` só é aceitável onde a invariante é garantida por este
código, e não pelo chamador — um literal de regex escrito neste arquivo,
uma variável de ambiente que o Cargo sempre define num build script — e a
mensagem do `expect` diz de qual invariante se está dependendo:

```rust
Regex::new(r"\[:([^:\]]+):\]").expect("built-in lang-tag pattern must compile")
```

---

## Dependências

Adicionar uma é uma decisão, não uma formalidade. A `numbertext` tem
exatamente uma dependência direta (`soros`), que por sua vez tem
exatamente uma (`fancy-regex`). É por isso que o parsing de argumentos do
CLI é feito à mão: as dependências de um alvo binário são as dependências
do pacote, então uma crate de parsing acabaria na árvore de todo mundo
que depende da biblioteca e nunca executa o binário.

Antes de adicionar uma dependência, pergunte o que ela custaria ao
consumidor, e se o código que ela substitui é genuinamente sutil o
bastante para valer a pena.

---

## Visibilidade

Público significa suportado. Tudo o que não faz parte da API de forma
deliberada é privado, e `pub(crate)` onde cruzar uma fronteira de módulo
dentro de uma crate for genuinamente necessário.

A divisão `soros`/`numbertext` é o mesmo princípio em escala de crate: o
interpretador é uma crate separada para que a fronteira seja imposta pelo
Cargo, e não por convenção. Mantenha a `soros` livre de qualquer conceito
de numbertext — se uma mudança fizer com que ela saiba o que é um
"ordinal", a mudança pertence ao outro lado da fronteira.
