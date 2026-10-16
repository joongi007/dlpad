# Windows CPU paged preview native CLI

[result](windows-cpu-paged-preview/result.json)와 각 원시 로그/manifest를 독립 대조했다. [독립 검토](windows-cpu-paged-preview/independent-review.json)는 clippy/test/build 각각258개 입력의 before=after=검토 시점 파일SHA 일치와 실제 exe를 확인한다. 정상 Git 저장소/remote CI는 없으며 이는 로컬 Windows GNU native CLI 결과다.

실제 실행은 소유 private desktop wrapper에서 다음 Cargo 작업을 수행했다. 정확한 Windows argv/RUSTFLAGS는 각 input manifest에 있다.

- `cargo clippy --workspace --all-targets --locked --features cpu -- -D warnings` exit0,15.304초.
- `cargo test --workspace --locked --features cpu` exit0,26.031초. test.log의 모든 terminal summary를 합산해250passed/0failed/1ignored를 확인했다.
- `cargo build --workspace --release --locked --features cpu` exit0,29.905초.

[test raw](windows-cpu-paged-preview/test.log), [clippy raw](windows-cpu-paged-preview/clippy.log), [release raw](windows-cpu-paged-preview/build.log)에 각각 input desktop unchanged=True가 남았다. GUI를 띄운 검증이라고 해석하지 않는다. 세 작업 모두 RUSTFLAGS `-C link-self-contained=yes`이며 pinned toolchain/Cargo.lock과 wrapper 격리 조건에서 실행했다.

현재 실제 `.local-tools/windows/target/release/dlpad-cpu.exe`는7244288bytes, SHA256 `a094b4ed7ef37b29f07e5c42d2af780e1d33d6408ac3cce5891c682012634ddb`로 result와 일치한다. 압축 설치 크기/외부 도구/working set 비용으로 이 uncompressed exe 크기를 대체하지 않는다.

공유 UI 입력은 Linux1669 snapshot의 paged preview/metrics/managedtools/history/background 연결을 포함한다. Windows CLI가 컴파일·회귀를 검증했지만 새 PLAN/최대report 페이지/clipboard/IME/reader/숨김 actualGUI를 실행한 것은 아니다. 예전 private Windows GUI의 숨김 증거도 해당 당시 snapshot 범위로 유지한다. Windows MSVC/macOS/패키지·설치·서명·최종UI스택S00 및 전체P0는 미완료다. ignored1개를 통과로 집계하지 않는다.


## 검토 뒤 테스트 범위 변경

위 독립 대조가 끝난 뒤 root가 download.rs의 순수 DB 이력 시험3개를 Unix-only에서 공통cfg(test)로 옮겼다. 제품 release 동작과 기존 Linux1669/Windowsa094 executable은 유지되지만 현재 source hash는 이전 Windows258 manifest의 download.rs 한 항목과 달라진다. 이전250PASS를 이 추가 플랫폼 시험의 성공으로 확대하지 않는다. 새 Windows CLI 재검증은 root가 별도 조정한다. independent-review.json의 current 비교는 변경 전 검토 시점 증거다.
