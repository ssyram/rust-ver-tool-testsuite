# 20 Tools — Upstream Citations (for paper draft)

> Each tool needs an academic citation in the paper. Estimates below — verify DOIs and final venue/year before camera-ready.

## Translation / Frontend Tools

### charon (charon-mono, charon-poly)

```
@inproceedings{ho2022aeneas,
  title={Aeneas: Rust Verification by Functional Translation},
  author={Ho, Son and Protzenko, Jonathan},
  booktitle={Proceedings of the ACM on Programming Languages (ICFP 2022)},
  year={2022},
  doi={10.1145/3547647}
}
```
Note: charon is the Rust → LLBC frontend of the Aeneas project; cite the Aeneas paper.

### aeneas (aeneas-coq, aeneas-fstar, aeneas-hol4, aeneas-lean)

Same as above — `ho2022aeneas`.

### rocq-of-rust (rocq-of-rust, rocq-of-rust-typecheck)

```
@misc{rocqofrust2024,
  title={rocq-of-rust: Translating Rust to Rocq},
  author={Formal Land},
  howpublished={\url{https://github.com/formal-land/rocq-of-rust}},
  year={2024},
  note={GitHub repository; locked to commit a8a76a4d}
}
```
No formal venue publication identified at the time of writing. Cite as a software artifact; locked-commit acceptable.

### hax (hax-coq, hax-fstar, hax-lean)

```
@misc{hax2024,
  title={hax: a tool for high-assurance translation of Rust},
  author={Cryspen et al.},
  howpublished={\url{https://github.com/hacspec/hax}},
  year={2024},
  note={Commit 30949eb87058895c24f963df90dd30ef11b0dc1a}
}
```
hax has a 2023 arXiv but the project recommends citing the GitHub project page. If the paper is finalized verify the latest hax citation.

## Verification Tools

### kani

```
@inproceedings{vanhattum2022kani,
  title={Kani Rust Verifier},
  author={VanHattum, Alexa and others (AWS Kani team)},
  booktitle={CAV},
  year={2022},
  note={Also see arXiv:2208.05545}
}
```
Verify exact CAV / Tool track entry; consult Kani team for canonical citation.

### prusti

```
@inproceedings{astrauskas2019leveraging,
  title={Leveraging Rust Types for Modular Specification and Verification},
  author={Astrauskas, Vytautas and M{\"u}ller, Peter and Poli, Federico and Summers, Alexander J.},
  booktitle={Proceedings of the ACM on Programming Languages (OOPSLA 2019)},
  year={2019},
  doi={10.1145/3360573}
}
```

### creusot

```
@inproceedings{denis2022creusot,
  title={Creusot: a foundry for the deductive verification of Rust programs},
  author={Denis, Xavier and Jourdan, Jacques-Henri and March{\'e}, Claude},
  booktitle={Verified Software: Theories, Tools and Experiments (VSTTE)},
  year={2022},
  doi={10.1007/978-3-031-25803-9_6}
}
```

### verus

```
@inproceedings{lattuada2023verus,
  title={Verus: Verifying Rust Programs using Linear Ghost Types},
  author={Lattuada, Andrea and Hance, Travis and Cho, Chanhee and Brun, Matthias and Subasinghe, Isitha and Zhou, Yi and Howell, Jon and Parno, Bryan and Hawblitzel, Chris},
  booktitle={Proceedings of the ACM on Programming Languages (OOPSLA 2023)},
  year={2023}
}
```

### verifast

```
@inproceedings{jacobs2011verifast,
  title={VeriFast: A Powerful, Sound, Predictable, Fast Verifier for C and Java},
  author={Jacobs, Bart and Smans, Jan and Philippaerts, Pieter and Vogels, Fr{\'e}d{\'e}ric and Penninckx, Willem and Piessens, Frank},
  booktitle={NASA Formal Methods},
  year={2011},
  doi={10.1007/978-3-642-20398-5_4}
}
```
The Rust frontend extension is in a 2023+ tech report — verify if newer authoritative cite exists.

### kmir

```
@article{rosu2010k,
  title={An overview of the K semantic framework},
  author={Ro{\c{s}}u, Grigore and {\c{S}}erb{\u{a}}nu{\c{t}}{\u{a}}, Traian Florin},
  journal={Journal of Logic and Algebraic Programming},
  volume={79},
  number={6},
  pages={397--434},
  year={2010},
  doi={10.1016/j.jlap.2010.03.012}
}
```
kmir applies the K framework to MIR. If a kmir-specific paper exists (Runtime Verification, Inc. publications), prefer that.

## Interpretive / Symbolic Execution

### miri

```
@inproceedings{jung2020stacked,
  title={Stacked Borrows: An Aliasing Model for Rust},
  author={Jung, Ralf and Dang, Hoang-Hai and Kang, Jeehoon and Dreyer, Derek},
  booktitle={Proceedings of the ACM on Programming Languages (POPL 2020)},
  year={2020},
  doi={10.1145/3371093}
}
```
Stacked Borrows is MIRI's core UB model; cite as semantic foundation. MIRI itself is in-tree of rust-lang/miri.

### soteria

```
@misc{soteria2024,
  title={Soteria: Symbolic Execution for Rust},
  author={Soteria Tools},
  howpublished={\url{https://github.com/soteria-tools/soteria-rust}},
  year={2024},
  note={Locked to commit 3c21278187c60c99418fe2dabb03710ce4102896}
}
```
No formal publication identified; cite as software artifact.

## Baseline

### cargo-check

```
@misc{cargo,
  title={Cargo: the Rust package manager},
  author={Rust Project},
  howpublished={\url{https://doc.rust-lang.org/cargo/}},
  year={2014--present}
}
```

---

## 总 cite 数

20 工具 → 11 distinct citations (charon + 4 aeneas-bk shares 1; rocq-of-rust + tier-1 shares 1; hax × 3 shares 1; charon × 2 shares 1).

## Paper 引用建议

- 主报告 §0.2 工具版本表，每行加 cite key 引用
- 主报告 §3 通过率表 caption 加 "tools cited in §0.2"
- Tool-by-tool 段（§5）首句加 inline cite

## 待确认（写 paper 时）

- kani 是否有 newer CAV/PLDI Tool Demo paper（2024+）
- hax 是否有最终 arXiv 或会议 paper
- verifast Rust 扩展是否有 dedicated 论文
- soteria 是否有 publication 计划
- rocq-of-rust 学术 publication（如有）
