# S04: 단일 Windows Job의 관리 실행·쿠키 paired config

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: 신규 crates/engine/src/process/windows_paired_guardian_config.rs, 이 문서
- 의존성: root process module 연결, 기존 PairedGuardianBinding 및 두 원 typed reader record, 후속 paired capture/runtime/entry

## 공유 API와 wire 계약

PairedGuardianConfig는 단일 executable/argv, managed_root_path/cookie_owner_path, 원 managed Intent/index/nonce 및 cookie Intent/index/nonce, timeout초/나노초, maxlogs를 가진다. CONFIG_DOMAIN은 pub(super) 고정 `dlpad-windows-paired-guardian-config-v1`이며 version1이다. validate/encode/decode/timeout 및 Windows process_spec/두 path getter를 제공한다. config는 256KiB JSON 상한·deny_unknown_fields·private wire와 custom Deserialize로 nested 검증을 강제한다. Debug/error는 고정 문자열이며 원 경로/argv를 로그로 내보내지 않는다.

validate는 PairedGuardianBinding::from_intents의 dummy creation(1,1)을 사용하여 원 record의 형식·anchor alias·정확 Metadata↔cookie0/Download↔cookie1 및 각 원 nonce 일치를 확인한다. 이미 Bound/Resolved거나 잘못된 stage/slot/domain을 거절한다. 이 dummy tuple은 실제 PID/creation/Job/파일·receipt 권한이 아니다. 경로는 NUL 없음·UTF16 units32766 상한을 검증하고 literal units를 보존하며 실제 absolute/native ID/ACL는 후속 entry/facade 담당이다. argv 256개/전체32766 units, timeout(0,24h], logs(0,MAX]를 유지한다.

## 인수와 실패 회귀

Metadata/Download roundtrip/redacted debug, exact 두 Intent/stage 매칭, timeout 나노초 및24h 경계, UTF16 길이/NUL/empty argv/총량 경계, direct와 nested serde의 unknownfield/domain/잘린JSON/anchor·nonce 변조/256KiB 초과 및 인코딩 초과를 시험한다. Windows literal ProcessSpec/path 변환 시험도 추가했다. 기존 cookie/managed config는 변경하지 않는다.

## 제품 수직 연결 범위

이 config만으로 단일 Job의 두 journal acquisition/receipt/Terminal이 완성되지 않는다. capture/entry/runtime이 원 capability HANDLE allowlist와 actual creation, 두 durable Bind, 두 독립 lock, 원 Job empty, 두 receipt sync를 연결해야 한다. 중간 cookie helper 우회 및 managed scope 분리 경로 제거는 root와 runtime 담당 후속 범위다. startup 자동 회수 완료를 주장하지 않는다.

## 검증 상태

개별 rustfmt exit 0. Cargo/부모 module/기존 config/schema 수정·실행은 하지 않았다. root의 실제 통합 검증 후 결과를 기록한다.

주 담당 Windows GNU CPU 전체 fmt·Clippy(-D warnings)·workspace test·release PASS: windows-paired-guardian-integrated-first. 최종 소스 SHA와 현재 파일을 독립 대조했다. 순수 binding/config/protocol 14개도 common cargo test로 통과했다. 실제 paired Job·두 receipt 성공/실패/취소 및 crash 인수는 아직 fixture 검증 대기다.
