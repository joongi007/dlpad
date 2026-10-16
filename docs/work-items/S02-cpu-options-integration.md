# S02: CPU typed 다운로드 옵션 통합

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R04, R06, R07, R08
- 담당/수정 범위: root; core/options.rs Quality 공통검증 API, accessible.rs, CPU/Glow main module 선언, catalog.rs, 본 문서. guardian_fault는 신규 accessible_options.rs 및 전용문서만.
- 선행 작업: S02-cpu-download-options, S02-cpu-settings-integration

## 사용자 결과

최고 품질/높이 제한/오디오 우선, 전체·구간/빠른·정확, 자막과 고급 옵션을 선택한다. 잘못된 값은 실행 전 거절하고 재개는 기존 snapshot을 유지한다. 긴 form은 스크롤하고 실행/중단 버튼을 항상 접근 가능하게 유지한다.

## 계약과 범위

DownloadForm Default/show/build와 core Quality::validate 공통정책 사용. Audio는bestaudio/best로 영상fallback 가능하고추출/형식변환을완료로표현하지않는다. 기존 field와IME는min_size32px로조정. 담당module 공유 API는root가조정하며schema/Cargo변경없음.

## 인수 조건

form→typedplan 경계·실패경로·snapshot 보존시험, fmt/clippy/workspace/release, privateGUI 실제선택과합성tool argv/Attempt/재개 일치확인. Windows/macOS 미검증은구분한다.

## 검증과 handoff

2026-10-03: Linux 후보의 한정된 수직 기능 통합 완료. 신규 모듈을 CPU/Glow entry에 연결했고 Quality::validate를 공유한다. 새 의존성·schema·Cargo.lock 변경은 없다. 오디오 우선은 추출/형식 변환이 아니며 실제 사이트 동작은 미검증이다.

실행한 최종 명령과 결과:

- cargo fmt --all -- --check: exit 0.
- cargo clippy --workspace --all-targets --locked -- -D warnings: exit 0.
- cargo test --workspace --locked: 197 passed, 5 ignored, 실패 0.
- cargo test --workspace --features cpu --locked: 280 passed, 5 ignored, 실패 0.
- cargo clippy -p dlpad --features cpu --all-targets --locked -- -D warnings: exit 0; CPU entry 시험 44 passed.
- cargo build --workspace --release --locked 및 CPU release 빌드: exit 0.
- Windows GNU 최종 snapshot: clippy/test/release 성공, 200 passed, 1 ignored. 주 담당이 입력 239개 해시와 실행 파일을 직접 대조했다. 실제 Windows 제품 GUI 검증을 뜻하지 않는다.

Linux CPU release SHA256: `2459ba126c6bcc13470b470c8a384d9d8ecef779893fd832411db0a78932f559`, 10,508,440 bytes. 설정 전용 f052 snapshot 대비 21,760 bytes 증가. 최신 옵션 snapshot의 RSS/CPU는 측정하지 않았으며 이전 설정 snapshot의 49.6 MiB/0.10%를 확대 적용하지 않는다.

[옵션 실제 GUI 검증](../validation/cpu-options-validation.md)의 최종 11건을 주 담당이 JSON/argv와 SQLite 사본 8개에 직접 대조했다. 성공 사본 integrity와 typed 옵션/Attempt 일치, 동일 Job의 Paused→Downloaded 및 원래 높이 제한 유지, 잘못된 높이/재시도/구간+자막 거절과 worker 미실행을 확인했다. 거절 사례는 DB가 생성되지 않아 존재하지 않는 DB의 행 수를 조회했다고 주장하지 않는다. 최초 수집 오류 2건과 수정 후 증거를 모두 보존했다. 실제 높이 제한 선택 PNG도 직접 검토했다.

[설정 GUI 검증](../validation/cpu-settings-validation.md): 최신 snapshot에서 입력 높이 100% 32px/200% 64px와 라이브러리 100% 32px, 테마 저장·복원·종료 대기 확인. 최신 IME PNG의 한[글] 조합 표시를 주 담당이 직접 관찰했으나 자동 preedit 판정 false 원본을 유지한다. 모든 단기 GUI 시험은 격리 화면에서 종료/정리했다.

실제 영상·사이트·오디오 추출·Windows/macOS 최신 GUI·전 배율 라이브러리 label/focus/루트 적용·최신 성능·전체 P0는 미검증 또는 미완료다. [현재 중단 지점](../validation/P0-status.md)을 다음 세션 기준으로 사용한다.
