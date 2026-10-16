# S04: 고정 source 검증·메모리 제한 XZ→TAR 연결

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R11
- 담당/수정 범위: root — tools.rs, tools/bounded_xz.rs, engine/Cargo.toml, Cargo.lock, ADR/이 문서; diagnostic_cache — 별도 tests/tar_xz_gate.rs/자체 작업 문서
- 선행 작업: bounded TAR reader 회귀/Clippy 통과, 실제GNU 원header 관측

## 사용자 결과

고정 압축 source를 검증한뒤 bounded native decoder와 TAR reader로 caller소유 output에 component를추출한다. 설치게시/활성화/공급trust를뜻하지않는다.

## 계약과 범위

lzma-rust2 =0.21.0 default-features=false features std,xz를 root가핀한다. 정확cache source의 new_mem_limit을확인했다. encoder/lzip/optimization/외부C라이브러리는사용하지않는다. 새sha2 0.11 계열의중복dependency비용이있으며최종binary크기/RSS는실제release측정후기록한다. LZMA2 decoder per-block memory상한64MiB(프로세스전체RSS상한아님),source256MiB/TAR별도limits64KiBstreambuffer와workerdeadline/cancel을적용한다. 원File SHA/size를decode전후확인하고singleXZstream뒤garbage/concat를거절한다. 원File의보관capability와caller의namespace/nativeidentity재확인은별개다. 새package/schema/Cargo workspace구조변경은없다.

## 인수 조건

실제합성정상/cancel/deadline/wronghash/truncation/corrupt/footer/tail/concat/메모리초과를시험하고source를보존한다. 원공식archive전체성공/성능/관리installer/Windows/macOS는후속실증이다.

## 검증과 handoff

정확candidate Cargo info와localsource API읽기완료. 제품dependency연결시작.

API 구현 후 `cargo test -p dlpad-engine --test tar_xz_gate --locked`4개PASS 및engine all-targetClippyexit0이다. root는compressed source자체SHA가각fixture에맞게제공되어decoder실패경로까지도달하는지검토했다. 추가lockpackage는lzma-rust2 0.21.0/sha2 0.11.0/digest0.11.3/crypto-common0.2.2/block-buffer0.12.1/hybrid-array0.4.15/const-oid0.10.2/cpufeatures0.3.1 총8개다. 전체source를메모리에읽지않고64KiBhashbuffer와원File reader로singleXZ전체소비/전후digest를확인한다. decoder의Interrupted재시도와취소를혼동하지않도록ownedcancel은고정Other오류뒤rootcancel분류를사용한다. 테스트의Windowsopenhandle cleanup과메모리초과typed근거를추가독립검토중이다.

## 공식 원 archive 실제 native 추출

root는release예제 SHA dd0b53a0768b33b32b3e5fa62c7bc8ddb68020f3197b5c2ae43e063d4b633ada/486280byte를실제로실행했다. 고정archive에서ffmpeg142534568B/ffprobe142305128B/LICENSE7651B각정확SHA를확인하고전후source SHA가같았다. 전체73member를판독하는3회stream은모두PASS이며툴실행·설치·활성화·재포장은없다. [원시 결과](../validation/evidence/ffmpeg-native-tar-xz-first/result.json). GNU/TAR전체통과를처음실증했으며이제관리공급채택이완료됐다는뜻은아니다.

최대RSS(/usr/bin/time)는10348KiB,wall32.13초이고제품Instant elapsed35073ms/외부Pythonelapsed35.11초를각각원시보존한다. 서로다른시간관측을하나의정밀값으로혼합하지않는다. component별대략10.6–11.2초이며각선택마다전체archive를다시판독해3회순회한다. 이486280B는독립검증예제전체크기이며UI앱증분크기나FFmpeg크기가아니다. 실사용installer연결의총임시용량·단일순회최적화·실제정확버전선택·trust/notice/3OS/fullworkspace는남는다.

강화된XZ5회귀(과대dictionaryOutOfMemory actualassert 포함)와engineall-targetClippy가terminalexit0이다. TempDir는원File/output종료뒤정리되며Windowsopenhandle정리문제를피한다. 최신WindowsGNU전체collector는실행중이며terminal전에는통과로집계하지않는다.

WindowsGNUCPU capability/fmt/clippy/test/release전체collector도terminalexit0/성공이며root가source_final의현재snapshotSHA불일치0을확인했다. 증거 docs/validation/evidence/tar-xz-windows-workspace-first/result.json. Linux전체collector를실제시작하여기존전체PASS와새8dependency/Rustsnapshot을구분한다. 실제WindowsFFmpeg공급과macOS/UI관리공급완료로확대하지않는다.
