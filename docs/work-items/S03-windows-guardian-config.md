# S03: Windows guardian bounded 설정

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: diagnostic_cache — 신규 crates/engine/src/process/windows_guardian_config.rs 및 이 문서
- 선행 작업: WindowsReaderRecord intent/bind 계약, 주 담당의 parent module/anonymous control pipe 연결

## 사용자 결과

Windows guardian에 전달하는 실행 설정을 제한된 크기로 검증하고 손상 설정·잘못된 reader intent를 실행 전에 거절한다. 실제 native guardian 실행 완료를 이 작업만으로 주장하지 않는다.

## 계약과 범위

version1 JSON은 unknown field를 거절하고 256KiB를 넘지 않는다. UTF-16 경로는 NUL 없이 각각 32766 units 이하, 인자는 256개 이하이며 합계 32766 units 이하다. 빈 인자는 허용한다. slot 0/1, 비영 nonce, 일치하는 Intent 상태만 허용한다. 시간은 0 초과 24시간 이하, 로그는 기존 MAX_LOG_BYTES 이하다. Debug와 오류는 고정 정제 문자열이다. Windows OsString 변환도 validate 이후 Result로 반환한다. 공유 모듈/Cargo/실제 pipe framing은 주 담당 범위다.

## 인수 조건

roundtrip 및 invalid version/slot/nonce/NUL/길이/시간/로그/unknown/oversize/Intent 상태·nonce 불일치가 순수 회귀 시험으로 확인된다. 경로·인자·비밀이 Debug/오류에 노출되지 않는다. Linux 순수 검증과 Windows 실제 실행 증거를 구분한다.

## 검증과 handoff

담당 모듈 구현 및 source freeze 완료. `rustfmt --edition 2024 crates/engine/src/process/windows_guardian_config.rs` exit0, `python3 scripts/check_docs.py` 494 documents PASS. 정상 Git 저장소 부재를 다시 확인했으며 기존 `.git`은 변경하지 않았다. 5개 순수 회귀 시험을 작성했으나 parent module 연결 및 Cargo 실행 결과는 아직 주 담당 통합 검증 대기다. 실제 Windows helper/process framing은 후속 통합 범위다. 순수 시험 작성이나 Linux 결과를 Windows 실행 성공으로 확대하지 않는다.

Root 모듈 연결 후 Linux `cargo test -p dlpad-engine --lib --locked windows_guardian_config` actualexit0/정확5PASS. Windows engine all-target Clippy에서도 실제컴파일exit0를확인했다. config전용native파서/runtime IPC 성공은 이결과로확정하지않는다.
