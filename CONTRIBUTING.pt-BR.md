# Contribuindo

*[Read in English](CONTRIBUTING.md)*

Este projeto segue o [Código de Conduta](CODE_OF_CONDUCT.md). Ao
participar, espera-se que você o siga.

## Antes de começar

Leia, nesta ordem:

1. [AGENTS.md](AGENTS.md) — arquitetura, a linguagem Soros condensada,
   limitações conhecidas/deliberadas, decisões de projeto não óbvias.
2. [docs/coding-style.pt-BR.md](docs/coding-style.pt-BR.md) — convenções
   de código além do que o `rustfmt` e o `clippy` já impõem.
3. [README.pt-BR.md](README.pt-BR.md) — a API voltada ao usuário e
   exemplos de uso.
4. [data/SOURCE.md](data/SOURCE.md) — de onde vêm os arquivos de locale
   `.sor` empacotados e como adicionar um novo.

## Ambiente

`make help` lista todos os comandos. Antes de abrir um PR:

```sh
make check     # fmt-check + lint + msrv + spell + testes (incl. doctests) + seleção de features
make lint-md   # lint de markdown (requer Node.js >= 20)
```

Esse é o mínimo que o CI executa em todo PR
([.github/workflows/ci.yml](.github/workflows/ci.yml)) — lá dividido em
jobs separados (`test` por versão do Rust, `lint` uma única vez, mais o
lint de markdown, o `cargo-deny` e a verificação de proveniência dos
dados) em vez de uma única chamada a `make check`. Veja a seção
"Commands" do AGENTS.md para entender por que essa divisão é deliberada e
não deve ser unificada.

Se a mudança alterar comportamento — e não for apenas refatoração —
inclua um teste cobrindo o novo caso. Para um idioma novo ou alterado,
adicione um punhado de conversões sabidamente corretas, **verificadas
executando o código de verdade** (`make run ARGS="--lang <x> --cardinal
<n>"`), não adivinhadas. Veja "Verify locale output before writing it
down" no AGENTS.md.

## Fluxo de branches

Rebase, não commits de merge: atualize sua branch com `git rebase` contra
a base antes de abrir ou atualizar um PR, em vez de mesclar a base na sua
branch. Histórico linear, sem commits de merge.

A convenção de commits abaixo é obrigatória na `main`. Uma branch de
trabalho que será esmagada (squash) no merge não precisa segui-la à
risca — o histórico intermediário não é o que permanece.

## Mensagens de commit

[Conventional Commits](https://www.conventionalcommits.org/), em inglês,
no modo imperativo:

```text
<type>(<scope>): <description>
```

* `<description>` no imperativo, descrevendo a ação (`add`, `fix`,
  `remove`, `harden`, `resolve`, `guarantee`), não o estado resultante
  (`added`) nem o histórico (`fixed`, "adds/added").
* `<scope>` é a crate ou área afetada. Não é uma lista fechada, mas os
  mais comuns são:

| Escopo | Abrange |
| --- | --- |
| `soros` | a crate do interpretador (`crates/soros`) |
| `numbertext` | a API pública, o `Locale`, o registro |
| `cli` | o binário `numbertext` (`src/bin`) |
| `data` | os arquivos de locale `.sor` empacotados e sua proveniência |
| `build` | `build.rs`, a tabela de locales gerada, features do Cargo |
| `docs` | prosa de README/AGENTS/CONTRIBUTING/skill |
| `ci` | `.github/workflows/` |
| `deps` | atualizações de dependências |
| `release` | incrementos de versão, cortes de changelog |

### Tipos

| Tipo | Quando usar |
| --- | --- |
| `feat` | Novo recurso (MINOR do semver). |
| `fix` | Correção de bug (PATCH do semver). |
| `docs` | Somente documentação (README, comentários de doc) — sem mudança de código. |
| `test` | Somente testes (adicionados, alterados ou removidos) — sem mudança de código de produção. |
| `refactor` | Muda como o código é escrito/organizado sem alterar o comportamento observável. |
| `perf` | Mudança cujo propósito é desempenho. |
| `style` | Formatação, espaços em branco, lint — sem mudança de código. |
| `build` | Build e dependências (`Cargo.toml`, `build.rs`, `Makefile`, ferramental). |
| `ci` | Integração contínua (`.github/workflows/`). |
| `chore` | Manutenção que não se encaixa nos tipos acima (config, `.gitignore`, ...). |
| `cleanup` | Remove código comentado, morto ou desnecessário — sem mudança de comportamento. |
| `remove` | Remove um arquivo, diretório ou recurso obsoleto/não utilizado. |
| `raw` | Mudança em arquivo de configuração/dados que não se encaixa acima (ex.: uma sincronização de `data/*.sor`). |

`make commit-check` verifica sua branch contra esta convenção
(`RANGE=origin/main..HEAD` para limitar aos seus próprios commits), e o
CI roda a mesma checagem em todo pull request. Os três tipos abaixo que
não fazem parte do Conventional Commits — `cleanup`, `remove`, `raw` —
estão declarados no `cog.toml`, então passam.

### Exemplos

```text
feat(numbertext): add Locale for resolving a language once
fix(soros): propagate the leading boundary through a piped call argument
docs(readme): explain regional variant fallback for en-GB/pt-BR
raw(data): sync data/*.sor to libnumbertext@a4b0225
```

## Mudando a API pública

O projeto está em **v0.x**, então uma mudança quebradora é permitida —
veja "Estabilidade da API" no README para o que isso significa para os
usuários. Ainda assim, ela precisa ser deliberada e visível:

* Diga isso na descrição do PR e adicione uma entrada `### Changed` ou
  `### Removed` no [CHANGELOG.md](CHANGELOG.md), com a migração na mesma
  entrada.
* Quando uma renomeação puder manter a grafia antiga compilando,
  mantenha-a como um alias `#[deprecated]` em vez de apagá-la de vez.
* Prefira uma mudança aditiva quando houver uma: um método novo ao lado
  do antigo, um parâmetro opcional novo via builder.

Elevar o MSRV é uma mudança quebradora sob as regras deste projeto, e
recebe o mesmo tratamento.

As flags do binário `numbertext` também contam como API pública — veja o
README. Renomear ou remover uma é uma mudança quebradora e passa por tudo
o que está acima, igual à assinatura de uma função.

O texto das mensagens de erro explicitamente **não** faz parte da API
pública; casar sobre ele não é suportado. As variantes de `Error` fazem.
O texto do `--help` também não.

## Trabalhando com os dados empacotados

`data/*.sor` vem do upstream e não é editado à mão — veja o AGENTS.md.
`make check-data` verifica essa afirmação contra os próprios hashes de
blob do upstream, e o CI a executa em todo PR, de modo que uma edição
avulsa falha visivelmente em vez de virar silenciosamente um fork
particular de um arquivo de regras.

Adicionar ou remover um arquivo de locale também exige `make
gen-features`: o Cargo não descobre features em tempo de build, então a
lista de features por idioma no `Cargo.toml` é gerada a partir do
diretório de dados.

## Contribuindo com um assistente de IA

Este projeto é construído com um, e não há nada a esconder ou declarar
sobre isso: o código é julgado da mesma forma de qualquer maneira, e um
PR não é marcado.

O que se pede é o mesmo que se pede a qualquer pessoa:

* **Entenda o que você está submetendo.** Se não souber explicar por que
  uma mudança está correta, ela não está pronta — independentemente do
  que a escreveu.
* **Verifique a saída do locale, não a adivinhe.** Um arquivo `.sor`
  codifica escolhas estilísticas reais e às vezes surpreendentes de cada
  idioma (veja os próprios exemplos do AGENTS.md: o "and" de `en` vs
  `en-GB`; a remoção do "and" em `cardinal` vs `year`). Execute o código
  antes de anotar um valor esperado.
* **Não deixe uma ferramenta reabrir uma decisão já tomada.** O AGENTS.md
  documenta as decisões de projeto deste port — o motor de regex de uma
  crate só, a alternativa de "sondar e verificar" que foi rejeitada, o
  patch de dados do `bg.sor` — porque cada fronteira foi alcançada uma
  vez, de propósito. Um PR que "conserta" uma delas adivinhando o
  raciocínio recebe um ponteiro, não um debate.
* **Rode as verificações localmente.** `make check` e `make lint-md`,
  antes de abrir o PR e não depois que o CI reclamar.

O AGENTS.md é lido diretamente pelo Codex e pelo Cursor; o CLAUDE.md o
importa para o Claude Code. Qualquer coisa que você adicionar para um
deles pertence ao AGENTS.md, para que os outros também recebam.

## Pull requests

* `make check` passando é obrigatório, não opcional. Se a mudança tocar
  em qualquer arquivo Markdown, `make lint-md` também.
* Um PR pequeno e focado em uma mudança é preferível a um PR grande
  cobrindo várias coisas não relacionadas — mas isso é bom senso, não
  regra estrita.
* Se a mudança alterar comportamento documentado, atualize a documentação
  no mesmo PR. Nos dois idiomas: uma edição em `README.md` que pule o
  `README.pt-BR.md` deixa o par se contradizendo.
