# S04: Windows paired guardian runtime

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 신규 process/windows_paired_guardian.rs 및 본 문서만
- 선행 작업: paired binding, 두 original guardian journal, root의 paired protocol/module 선언

## 사용자 결과

관리 도구와 세션 쿠키를 사용하는 한 suspended child를 원 Job 안에서 실행하며 두 journal의 종료 증거를 확인한다.

## 계약과 범위

Request는 두 Intent/index/nonce/path/원 receipt File과 단일 control/events/stdout/stderr를 받는다. 실제 creation times로 pairdigest를 만든다. Prepared→parent Ack→managed acquire→cookie acquire→pair Ack→Begin→Resume 순서다. 별도 protocol magic은 root가 제공한다. 기존 guardian/Cargo/공유 파일은 수정하지 않는다.

## 인수 조건

두 원 journal 검증 전 실행하지 않는다. 원 Job terminal 확인 뒤 두 complete가 성공하고 journal handle이 닫힌 다음에만 pair Terminal을 전송한다. 부분 acquire/complete 실패는 두 완료를 주장하지 않으며 pending을 보존한다. partial pipe, EOF, deadline 회귀를 Windows 실제 pipe로 검증한다.

## 검증과 handoff

런타임 구현 및 개별 rustfmt 완료. 실제 Windows 컴파일/native 시험은 주 담당 검증 대기다. 별도 paired protocol module을 요구한다. 원 pipe의 partial/EOF/deadline, malformed/cross-domain frame 거절 및 nonce/slot/kind/digest 대조 테스트를 작성했다. 단위 pipe 시험은 실제 paired Job 성공이나 startup 회수를 대신하지 않는다.

주 담당 Windows GNU CPU 전체 fmt·Clippy(-D warnings)·workspace test·release PASS: windows-paired-guardian-integrated-first. 최종 소스 SHA와 현재 파일을 독립 대조했다. 순수 binding/config/protocol 14개도 common cargo test로 통과했다. 실제 paired Job·두 receipt 성공/실패/취소 및 crash 인수는 아직 fixture 검증 대기다.

실제 paired 성공 인수: windows-paired-public-download-duration-retry에서 공개 Repository의 버전 확인 및 인증 metadata/download를 실행했다. 한 원 Job의 두 journal·receipt 확인, 인증 출력 생략, 원 cookie/hash·sentinel 보존, 원 소유 기록 정리·재사용을 통과했다. 주 담당이 job_json의 Downloaded/artifact, DB/WAL/SHM의 합성 SECRET 부재, 원 cookie SHA, cookie cache에 foreign sentinel만 남음, managed 9파일/pending 부재와 현재 소스/고정 binary SHA를 독립 대조했다. 실패·취소·부모 crash 및 startup 자동 회수는 아직 후속이다. 마지막 전체 Windows GNU CPU fmt/clippy/test/release도 windows-paired-native-fixture-workspace에서 통과했다.
