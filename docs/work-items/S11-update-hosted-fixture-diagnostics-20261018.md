# S11: Hosted updater fixture 실패 관측

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R10, R13
- 담당/수정 범위: update_transport — `crates/engine/src/bin/app_update_fixture.rs`, 이 문서
- 선행 작업: S11-update-journal-20261018, immutable v0.1.1 hosted fixture 실패 원본

## 사용자 결과

릴리스 CI에서 합성 updater 전환이 실패하면 분리된 자식의 고정 단계·실패 분류가 원 artifact stderr에 남아 원인을 판별할 수 있다. 제품 업데이트 정책·신뢰키·UI·설치 동작은 변경하지 않는다.

## 계약과 범위

Root가 공유 Cargo/CI/버전·태그를 담당한다. v0.1.1 태그는 옮기지 않는다. 신규 기본 OFF fixture bin만 수정하며 제품 host/journal/IPC/schema를 변경하지 않는다. 운영 키·계정·실제 영상은 사용하지 않는다.

실패 시 owner Drop이 고정 단계, 알려진 marker, worker/request 수, 자식 관측을 stderr `FIXTURE_DIAGNOSTIC` JSON으로 기록한다. root 최상위 scan128개·자식 record32개·각2KiB로 제한한다. 원래 임시 root와 실패 파일은 보존한다. arbitrary error 문자열과 실제 root path는 출력하지 않는다. IO kind/raw OS code, profile enum, generic fixture class 및 고정 ParentFailed code/generation만 허용한다. child_entry 자체 실패도 environment에 전달된 합성 root의 exact sentinel·regular/reparse 정책을 확인한 뒤 진단한다. 성공 시 기존 CASE_PASS 계약과 자기 root 정리를 유지한다.

## 인수 조건

- 원 자식/부모 단계로 initial bootstrap, 준비 중 ParentFailed, 재시작, cold/replay 실패를 구분한다.
- source checkout fixture 전용 strict Clippy·진단 회귀와 합성 Windows 동작을 검증한다.
- 인증/경로를 포함한 임의 오류 문자열이 관측 payload에 섞이지 않는다.
- native 시험은 기존 소유 Job·headless fixture 범위를 유지하며 사용자 입력 desktop을 변경하지 않는다.

## 검증과 handoff

원 hosted capture는 case-success exit1, journal generation0/highest0, stdout0, natural Job0·강제정리0였다. 기존 stderr의 generic 오류로 최초 A 또는 요청 후 실패를 구별할 수 없었다. 이것을 특정 Server2022 원인으로 확정하지 않았다. 구현 파일에 rustfmt를 적용했으며 Windows compile/lint/진단 회귀와 hosted 실제 재실행은 아직 검증 중이다.

Root 통합 검증: Windows 원 소유 Job에서 `cargo test -p dlpad-engine --bin app-update-fixture --features app-update-fixture --locked --offline` 진단2 PASS, 기존 wrapper strict Clippy/build와 실제 전환7case 전부 PASS. 원 Job natural0/forcedfalse/입력 desktop 불변. 원본은 `.local-tools/update-publication-20261018/native-ci-fixture-v2/native-tests.log`, `.local-tools/update-activation-20261018/ci-fixture-v2/whole.json`. `python3 scripts/check_docs.py --source-only` 1139문서 PASS; wrapper 정책11/패키징13 PASS. hosted 원인 판별은 다음 실행 대기.
