# numbertext-rs

[![CI](https://github.com/casara/numbertext-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/casara/numbertext-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/numbertext.svg)](https://crates.io/crates/numbertext)
[![docs.rs](https://img.shields.io/docsrs/numbertext)](https://docs.rs/numbertext)
[![Licença: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Uma biblioteca Rust que escreve números por extenso — cardinais,
ordinais, variantes de gênero/caso, negativos e valores monetários — em
todos os idiomas distribuídos pela
[Numbertext.org](https://numbertext.github.io/).

Read this in [English](README.md).

## Por quê

A [Numbertext.org](https://numbertext.github.io/) (upstream:
[libnumbertext](https://github.com/Numbertext/libnumbertext)) descreve a
conversão de número para texto de dezenas de idiomas como pequenos
arquivos de regras declarativos, escritos numa DSL minúscula de reescrita
por regex chamada **Soros**, em vez de código escrito à mão para cada
idioma. Adicionar um idioma — ou uma variante regional ou de gênero de um
já existente — significa escrever um novo arquivo `.sor`, não escrever
código novo.

Este repositório é uma implementação em Rust, feita do zero, do
interpretador Soros (a crate [`soros`](crates/soros)) mais uma API
pública amigável (a crate [`numbertext`](src/lib.rs)), reaproveitando sem
modificação os mesmos arquivos de regras `.sor` do upstream (veja
[`data/`](data) e [`data/SOURCE.md`](data/SOURCE.md)).

## Instalação

Como biblioteca:

```sh
cargo add numbertext
```

Como ferramenta de linha de comando:

```sh
cargo install numbertext
```

Isso instala um binário `numbertext` construído a partir desta mesma
crate, com todos os idiomas empacotados:

```sh
numbertext --lang en --cardinal 12345    # twelve thousand three hundred forty-five
numbertext --lang pt-BR --ordinal 21     # vigésimo primeiro
numbertext --lang de --year 1999         # neunzehnhundertneunundneunzig
numbertext --list                        # todos os códigos de idioma embutidos
numbertext --help                        # todas as ações e opções
```

O binário não custa nada a quem usa a biblioteca — o Cargo não compila os
alvos binários de uma dependência. Ele também respeita as features de
idioma descritas abaixo, então `cargo install numbertext
--no-default-features --features lang-en` entrega uma ferramenta só com
inglês.

### Completions de shell

O diretório [`completions/`](completions) traz scripts para bash, zsh e
fish. Eles não são gerados em tempo de compilação, então o
`cargo install` não os instala por você — copie o que precisar:

```sh
# bash
. completions/numbertext.bash                      # ou carregue a partir do ~/.bashrc

# zsh — o nome do arquivo importa, é por ele que o zsh faz o autoload
mkdir -p ~/.zfunc && cp completions/_numbertext ~/.zfunc/
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc

# fish
cp completions/numbertext.fish ~/.config/fish/completions/
```

O `--lang` completa a partir de `numbertext --list` no próprio binário
sendo completado, então ele oferece exatamente os idiomas que *aquele*
build embutiu — uma instalação só com inglês completa apenas `en`, sem
uma segunda lista para manter sincronizada. A lista de flags é conferida
contra o próprio `--help` do CLI pelo `tests/completions.rs`, então uma
flag adicionada num lado e não no outro quebra o build.

## Uso

```rust
use numbertext::Locale;

fn main() -> Result<(), numbertext::Error> {
    let en = Locale::new("en")?;

    println!("{}", en.cardinal("12345")); // twelve thousand three hundred forty-five
    println!("{}", en.ordinal("21"));     // twenty-first
    println!("{}", en.year("1999"));      // nineteen ninety-nine
    println!("{}", en.currency("USD", "2.50")); // two U.S. dollars and fifty cents

    let pt = Locale::new("pt")?;
    println!("{}", pt.cardinal("-5"));    // menos cinco

    Ok(())
}
```

### Dois caminhos de entrada

`Locale` resolve e compila um idioma uma única vez. Depois disso, **a
conversão não pode falhar** — todo método devolve uma `String` simples,
porque o motor subjacente não tem noção de conversão malsucedida (uma
entrada que nenhuma regra casa produz uma string vazia). Prefira esse
caminho sempre que converter mais de um número no mesmo idioma.

As funções livres recebem o código do idioma a cada chamada e embutem a
busca no tipo de retorno, para uso pontual:

```rust
let n = numbertext::cardinal("de", "42")?; // "zweiundvierzig"
# Ok::<(), numbertext::Error>(())
```

Ambos passam pelo mesmo cache global do processo, então nenhum dos dois
recompila um idioma já utilizado.

### API

| Método de `Locale` | Função livre | Finalidade |
| --- | --- | --- |
| `cardinal(n)` | `cardinal(lang, n)` | "123" → "one hundred twenty-three" |
| `cardinal_i64(n)` | — | o mesmo, para um `i64` |
| `ordinal(n)` | `ordinal(lang, n)` | "123" → "one hundred twenty-third" |
| `ordinal_number(n)` | `ordinal_number(lang, n)` | "123" → "123rd" |
| `year(n)` | `year(lang, n)` | "1999" → "nineteen ninety-nine" |
| `currency(iso, amount)` | `currency(lang, iso, amount)` | estilo `NUMBERTEXT()`: "USD", "2.5" → "two U.S. dollars and fifty cents" |
| `money(iso, amount)` | `money(lang, iso, amount)` | estilo `MONEYTEXT()`: "USD", "2.5" → "two and 50/100 U.S. dollars" |
| `help()` | `help(lang)` | a seção de uso autodocumentada do locale |
| `convert(prefix, arg)` | `convert(lang, prefix, arg)` | a primitiva sobre a qual tudo é construído — veja abaixo |
| `skipped_rules()` | — | linhas de regra que não compilaram (vazio em todo locale embutido) |
| — | `register_locale(code, source)` | adiciona um idioma em tempo de execução a partir de texto `.sor` bruto |
| — | `languages()` | todos os códigos de idioma conhecidos no momento |
| — | `language_variants()` | os códigos de variante declarados dentro desses arquivos |

Todos os argumentos numéricos são `&str` (`"123"`, `"-5"`, `"3.14"`), não
inteiros nem floats, porque o motor subjacente é baseado em texto e isso
mantém exatos os números de precisão arbitrária — um arquivo `.sor` pode
escrever por extenso números muito além de qualquer tipo de largura fixa.
`cardinal_i64` cobre o caso comum de inteiros.

`currency` e `money` são duas representações distintas do mesmo valor,
definidas pelo upstream, e não grafias intercambiáveis. Escolha a que o
seu formato de saída realmente espera.

### Variantes regionais (ex.: "en-GB", "pt-BR")

Um código regional não precisa — e não terá — um arquivo `.sor` próprio:
`data/en.sor` e `data/pt.sor` carregam as regras do idioma base mais
algumas linhas marcadas com `[:en-GB:]`, `[:pt-BR:]` e afins, para a
redação que varia por região. Basta passar o código completo:

```rust
# use numbertext::Locale;
# fn main() -> Result<(), numbertext::Error> {
let gb = Locale::new("en-GB")?;
let us = Locale::new("en")?;

assert_eq!(gb.cardinal("101"), "one hundred and one");
assert_eq!(us.cardinal("101"), "one hundred one"); // sem "and" — a redação padrão

let br = Locale::new("pt-BR")?;
let pt = Locale::new("pt")?;

assert_eq!(br.cardinal("16"), "dezesseis");
assert_eq!(pt.cardinal("16"), "dezasseis"); // padrão do português europeu
# Ok(())
# }
```

`languages()` lista apenas os códigos que têm arquivo próprio (`"en"`,
`"pt"`, ...); `language_variants()` lista os declarados por linhas
marcadas dentro deles (`"en-GB"`, `"pt-BR"`, ...). O `--list` do CLI
imprime os dois, e é isso que as completions de shell leem.

Nenhuma das duas listas esgota o que o `Locale::new` *aceita*: qualquer
`"<base-conhecida>-<qualquer-coisa>"` resolve para o arquivo base —
`"en-ZZ"` devolve inglês comum em vez de erro. O que as listas contêm é o
conjunto de códigos que de fato mudam a saída.

Duas regras de resolução valem ser conhecidas, ambas iguais às do
upstream:

* Uma variante de escrita é nomeada com `_` no arquivo, mas pode ser
  escrita com qualquer separador: `"hu_Hung"` e `"hu-Hung"` dão os dois
  húngaro antigo.
* O norueguês é um caso especial — `"nb"`, `"nn"`, `"nb-NO"` e `"nn-NO"`
  leem todos o `no.sor`, onde os dois padrões escritos convivem.

### Gênero, caso e outras variantes específicas do idioma

Os arquivos de regras da Numbertext não se limitam a um formato fixo de
cardinal/ordinal — um idioma pode definir seções nomeadas arbitrárias
para quaisquer variantes gramaticais de que precise (`pt.sor` tem
`feminine`/`masculine`; `ru.sor` tem `cardinal-feminine`/
`cardinal-neuter`; `it.sor` tem `ordinal-masculine`, e assim por diante).
Esses nomes de seção **não** são padronizados entre os idiomas, então
esta biblioteca não finge o contrário com um método `gender(...)`
único para todos os casos. Chame `convert` com o nome da seção
desejada:

```rust
# use numbertext::Locale;
# fn main() -> Result<(), numbertext::Error> {
let pt = Locale::new("pt")?;

assert_eq!(pt.convert("feminine", "2"), "duas");
# Ok(())
# }
```

Execute `help(lang)`, ou abra o arquivo `.sor` diretamente, para ver
quais seções um dado idioma define.

### Escolhendo quais idiomas são embutidos

Por padrão, todos os 52 arquivos `.sor` empacotados são embutidos no seu
binário, independentemente de quais idiomas o seu programa realmente
chama. Se você só precisa de alguns, opte por um build menor usando
features do Cargo — uma `lang-<código>` por idioma, onde `<código>` é o
nome base do arquivo `.sor` em minúsculas, com `_` virando `-` (`en`,
`pt`, `hu-hung`):

```toml
[dependencies]
numbertext = { version = "0.1", default-features = false, features = ["lang-en", "lang-pt"] }
```

Isso embute apenas `en` e `pt`: `languages()` devolve exatamente
`["en", "pt"]`, as variantes regionais desses dois idiomas (`en-GB`,
`pt-BR`) continuam funcionando, já que vivem dentro desses mesmos
arquivos, e chamar `cardinal("de", ...)` devolve
`Error::UnknownLanguage`. A economia é de cerca de 236 KB de dados de
regras embutidos — mensurável em relação aos dados, não em relação ao seu
binário inteiro.

Repare no modo de falha: **é silencioso em tempo de compilação e ruidoso
em tempo de execução.** Selecionar o conjunto errado ainda compila; o
idioma simplesmente não está lá quando você o chama. Se adicionar uma
chamada para um idioma novo, adicione também a feature dele.

`default-features = false` sem nenhuma feature `lang-*` não embute nada
(`languages()` devolve uma lista vazia). Isso só é útil se você fornecer
todos os locales por conta própria via `register_locale`.

As features do Cargo são aditivas ao longo do grafo de dependências, o
que se comporta bem aqui: se duas crates do seu build pedirem conjuntos
de idiomas diferentes, você recebe a união, nunca um idioma faltando.

### Adicionando um idioma novo

Como a biblioteca inteira é orientada a dados, não é preciso tocar em
nenhum código Rust para adicionar um idioma:

```rust
let source = "1 uno\n2 dos\n3 tres\n"; // um arquivo real precisa de muito mais regras
numbertext::register_locale("es-mini", source)?;

assert_eq!(numbertext::cardinal("es-mini", "2")?, "dos");
# Ok::<(), numbertext::Error>(())
```

ou adicione um arquivo em [`data/`](data) — veja
[`data/SOURCE.md`](data/SOURCE.md#adding-a-new-language) para um guia
curto e um link para a especificação da linguagem Soros do upstream.

## O interpretador Soros

A crate [`soros`](crates/soros) é o interpretador da linguagem por si só:
sem nenhuma noção de números, moedas ou idiomas. É uma crate separada, e
não um módulo privado, para que a fronteira seja imposta pelo Cargo em
vez de por convenção — a `soros` não tem dependência de `numbertext` pela
qual pudesse voltar. Ela é utilizável diretamente, caso você tenha algum
outro uso para a linguagem:

```rust
let program = soros::Program::compile(r"(.*)(.) \2$1", "");

assert_eq!(program.run("hello"), "olleh");
```

Os padrões compilam via [`fancy-regex`](https://docs.rs/fancy-regex), que
usa o motor de tempo linear [`regex`](https://docs.rs/regex) para
qualquer padrão sem backreference ou lookaround, e um motor de
backtracking limitado por número de passos para o restante — do qual
alguns arquivos de locale reais genuinamente precisam (`hu.sor` consulta
sufixos gramaticais com uma backreference dentro do padrão de casamento).
Veja [AGENTS.md](AGENTS.md#regex-engine-one-crate-not-two-glued-together)
para o raciocínio completo.

## Estabilidade da API

A `numbertext` é publicada como **v0.x** e, no Versionamento Semântico,
isso carrega um significado específico que vale declarar explicitamente
em vez de deixar você inferir: **uma versão menor pode quebrar a API
pública.** Fixe uma versão, leia o [CHANGELOG](CHANGELOG.md) antes de
atualizar e espere fazer pequenas mudanças mecânicas quando o fizer.

O que conta como API pública: todo item público das crates `numbertext` e
`soros`, **e as flags de linha de comando do binário `numbertext`** — a
partir do momento em que uma ferramenta é instalável, as flags são um
contrato quer alguém diga isso ou não, então aqui está dito.

Não é público: o texto exato das mensagens de erro, da saída de `--help`
e da saída de `help()` (que é prosa do upstream e muda a cada
sincronização de dados). As variantes de `Error` — feitas para serem
casadas por padrão — são a parte estável do tratamento de erros; o texto
da mensagem não é.

O MSRV é **Rust 1.85**, e o CI testa exatamente essa versão além da
`stable`, para que a declaração continue honesta. Elevá-lo é uma mudança
quebradora e recebe uma entrada no changelog.

Mudanças quebradoras são listadas em `### Changed` ou `### Removed` no
changelog, com a migração na mesma entrada.

## Skill para assistentes de IA

O arquivo
[skills/using-numbertext/SKILL.md](skills/using-numbertext/SKILL.md)
documenta, num formato que ferramentas de IA carregam sob demanda (Claude
Skill), como usar a `numbertext` de forma idiomática: `Locale` versus as
funções livres, a primitiva `convert`, variantes regionais, seções de
gênero/caso e a seleção de idiomas embutidos. Para usá-lo num projeto que
depende da `numbertext`, copie o diretório para dentro dele:

```sh
cp -r skills/using-numbertext <seu-projeto>/.claude/skills/using-numbertext
```

## Desenvolvimento

`make help` lista todos os comandos. Antes de abrir um PR:

```sh
make check     # fmt-check + lint + msrv + spell + testes (incl. doctests) + seleção de features
make lint-md   # lint de markdown (requer Node.js >= 20)
```

As convenções de estilo estão documentadas em
[docs/coding-style.pt-BR.md](docs/coding-style.pt-BR.md).

Veja [CONTRIBUTING.pt-BR.md](CONTRIBUTING.pt-BR.md) para o fluxo completo
de contribuição (convenção de mensagens de commit, processo de
branch/PR) e [CLAUDE.md](CLAUDE.md)/[AGENTS.md](AGENTS.md) para uma visão
mais profunda da arquitetura, voltada a quem contribui e a agentes de
programação.

## Licença

O código Rust deste repositório é licenciado sob MIT, veja
[LICENSE](LICENSE).

Os arquivos de regras `.sor` em [`data/`](data) são outra história: eles
são copiados sem modificação do libnumbertext upstream e permanecem sob a
licença **BSD-3-Clause** dele, veja [`data/LICENSE`](data/LICENSE) e
[`data/SOURCE.md`](data/SOURCE.md). As duas licenças acompanham a crate,
já que os arquivos de regras estão embutidos nela.
