# S02: Windows 인증 다운로드 실제 engine fixture

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: soak_recovery_audit — 신규 crates/engine/examples/session_cookie_fixture_tool.rs, 신규 crates/engine/tests/windows_session_cookie_download.rs 및 이 문서; root — Cargo/native build·실행 및 GUI 연결
- 선행 작업: Windows cookie4/Job3/pipe3 actual시험, candidateworker 연결

## 사용자 결과

Windows에서실제DownloadRepository가metadata와download에같은privatecopy를전달하고version에는전달하지않는지검증한다. 확인된완료·실패·취소뒤정리와로그/metadata/DB의비밀생략을확인한다.

## 계약과 범위

Rust dev example만합성yt-dlp대역으로만들고제품binary배포에포함하지않는다. 새Cargo/dependency/schema없음. 고정synthetic쿠키만허용하여실제credential을읽는도구로쓰지않는다. stdout/metadata/로그/통신기록은bounded이고source/fixtureowner는임시폴더만사용한다. shell문자열실행없음. 시험은명시적 DLPAD_WINDOWS_COOKIE_FIXTURE_TOOL nativeexepath가주어진경우에만실행하며무설정은ignored별도기록한다. 원본/sentinel/copy확인과artifact정합성,새Job/Attempt정책을실제DB에서검증한다.

## 인수 조건

version쿠키0,metadata·download동일literalcopy,successfulartifact/DB재열기,실패·취소미완료상태및원본보존,copy소유정리,보고서/DBprivatepath·value없음. root가Windowsnativeexample을별도빌드하고실제시험을실행한다. Rust단위시험/CLI를GUI·실제사이트인수로대체하지않는다.

## 검증과 handoff

agent sourcefreeze후root가argv형태의ownedWindowsCLIhost로실행한다. 기존cookie/process/UI/sharedCargo파일은동시수정하지않는다.

## 완성 fixture protocol과 sourcefreeze

새 example/test의 소유 rustfmt 완료 후 root 인수 대기다. `DLPAD_WINDOWS_COOKIE_FIXTURE_TOOL`은 별도로 빌드한 native example 절대 경로다. 하나의 ignored `native_authenticated_download_success_failure_and_cancel` test가 success/metadata-failure/download-failure/cancel 4mode를 순차 실행하며, 매 mode마다 OS temp 바로 아래 `dlpad-windows-cookie-download-owned-*` TempDir에 example을 복사한다. Env global mutation과 shell 문자열 실행은 없다.

Example은 current_exe parent prefix·OS temp parent·고정 `fixture.cap`·exact synthetic source를 확인하고, reparse/symlink를 거절한다. Version에는 쿠키가 없어야 한다. Metadata/download에는 그 root의 private-cache 아래 exact fixed Netscape bytes만 허용한다. 4096-byte read/128argv/32768bytes per argv/65536bytes total argv, calls JSONL 16384byte 기존 상한을 둔다. Calls에는 stage/private path/literal SHA만 기록하고 cookie 값을 저장하지 않는다. 고정 가짜 secret/path를 raw stdout/stderr/metadata에 의도 echo하여 authenticated omission을 시험한다. Metadata failure exit7/download failure exit9, cancel은 소유 partial와 worker-ready를 만든 뒤30초 대기한다. Success artifact는 소유 paths 아래 새 파일로만 생성한다. 새 dependency/feature/제품 배포 파일은 없다.

Test는 실제 DownloadRepository/Windows process ownership을 사용하며 version/metadata/download calls와 동일 copy SHA, 정상 artifact bytes, 민감 output/free metadata 생략, DB job/options/rawbytes와 재열기의 비노출 및1Attempt, 실패 Failed/취소 Cancelled, reader 재예약·confirm·cleanup/copy directory 제거/source/sentinel 보존을 확인한다. SQLite 2MiB/calls32KiB를 제한한다. 테스트는 실제 계정·외부 사이트·GUI를 인수하지 않는다.

Root 실행 계약(Windows native toolchain/hidden CLI host에서):

```text
cargo build -p dlpad-engine --example session_cookie_fixture_tool --locked
DLPAD_WINDOWS_COOKIE_FIXTURE_TOOL=<native example absolute exe>
cargo test -p dlpad-engine --test windows_session_cookie_download --locked -- --ignored --exact native_authenticated_download_success_failure_and_cancel --nocapture
```

하위 담당은 Cargo/Windows 실행을 하지 않았다. Compile/runtime/clippy는 아직 미검증이며 root가 최신 source snapshot을 고정한 뒤 실제 실행한다.

## 통합 담당 실제 검증

2026-10-04 root가 Windows GNU native example 빌드와 exact ignored 통합 시험을 실행했다. 두 명령 exit0, 입력 소스 before/after 및 root 재검토 일치. 성공·metadata 실패·download 실패·취소 4mode 모두 PASS, 사용자 input desktop unchanged. 증거: `docs/validation/evidence/windows-authenticated-download-first/`의 build/test log·inputs·root-review.json. 실제 사이트·GUI·재시작 복구는 이 결과 범위에 포함하지 않는다.
