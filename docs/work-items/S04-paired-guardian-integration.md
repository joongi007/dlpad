# S04: 단일 Job의 관리·인증 보호 결합

- 상태: in-progress
- 분류: safety-integration
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: root: paired protocol/entry, helper launcher, native spawn allowlist, 공유 선언과 제품 호출; 하위 담당: 별도 config/runtime/capture 파일
- 선행 작업: 관리 공개 다운로드 인수, cookie guardian, paired binding 순수 계약

## 사용자 결과

관리 도구를 인증과 함께 실행할 때 단일 원 Job의 두 보호 기록과 receipt를 모두 확인하여 실행·완료·정리한다.

## 계약과 범위

별도 DLPPG001 제어 magic, 정확히 6개 원 capability만 전달한다. 관리 및 쿠키 receipt는 서로 다른 원 File이다. 두 Bound 동기화와 독립 journal 잠금 확인 뒤에만 Begin을 허용한다. 원 Job 종료 뒤 두 receipt가 모두 동기화된 경우에만 Terminal을 인정한다. 부분 실패는 보존한다. 기존 관리 전용/인증 전용 5개 capability 계약을 유지한다.

## 인수 조건

변조·다른 domain·receipt 하나 누락·실행 취소·metadata/download 실제 실패·부모 종료를 합성 fixture와 격리 Windows에서 검증한다. 단위 시험을 실제 Job 두 기록 종료 권한으로 확대하지 않는다. 원 사용자 인증이나 영상은 사용하지 않는다.

## 검증과 handoff

구현 및 실제 회귀 결과를 후속 기록한다. startup 자동 회수는 전체 reader 연결 인수 후에만 활성화한다.

주 담당은 helper의 Prepared digest를 기존 canonical 준비 프레임의 zero 규칙에 맞췄다. 실제 creation을 양쪽이 확인한 뒤 두 Bound의 pair digest를 Ack/Begin/Terminal에서 대조한다. 전달받은 구현의 Prepared에 pair digest를 넣은 불일치는 OS 시험 전에 통합 검토에서 수정했다.

제품 managed+cookie metadata/download는 기존 단독 cookie fallback 대신 새 paired capture로 전환했다. 같은 원 Tool Job의 두 기록 종료 확인을 run_paired_tool 실행 capability로 연결한다. 이전 중간 fallback 상태는 이 구현으로 대체하지만 실제 native 인수 전 완료로 표시하지 않는다. startup 자동 회수는 아직 비활성이다.

주 담당 Windows GNU CPU 전체 fmt·Clippy(-D warnings)·workspace test·release PASS: windows-paired-guardian-integrated-first. 최종 소스 SHA와 현재 파일을 독립 대조했다. 순수 binding/config/protocol 14개도 common cargo test로 통과했다. 실제 paired Job·두 receipt 성공/실패/취소 및 crash 인수는 아직 fixture 검증 대기다.

실제 paired 성공 인수: windows-paired-public-download-duration-retry에서 공개 Repository의 버전 확인 및 인증 metadata/download를 실행했다. 한 원 Job의 두 journal·receipt 확인, 인증 출력 생략, 원 cookie/hash·sentinel 보존, 원 소유 기록 정리·재사용을 통과했다. 주 담당이 job_json의 Downloaded/artifact, DB/WAL/SHM의 합성 SECRET 부재, 원 cookie SHA, cookie cache에 foreign sentinel만 남음, managed 9파일/pending 부재와 현재 소스/고정 binary SHA를 독립 대조했다. 실패·취소·부모 crash 및 startup 자동 회수는 아직 후속이다. 마지막 전체 Windows GNU CPU fmt/clippy/test/release도 windows-paired-native-fixture-workspace에서 통과했다.
