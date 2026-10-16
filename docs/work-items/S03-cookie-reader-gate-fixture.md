# S03: cookie reader gate의 합성 crash fixture

- 상태: in-progress
- 분류: test-support
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: stress_recovery; 신규 scripts/cookie_reader_gate_fixture.py 및 이 문서
- 선행 작업: root 실제 reader B gate/receipt driver 연결

## 사용자 결과

고유 /tmp namespace에서 합성 쿠키 source와 sentinel을 만들고 metadata/download 두 슬롯의 도구 실행 및 B 전후 관측을 구분한다.

## 계약과 범위

create-only 최대12파일/전체64KiB fixture, file0600/root0700, 절대 /tmp 경로와 symlink 거절이다. 네트워크/server/systemd/신호/restart/Cargo/제품/기존8h 수정은 없다. create ROOT --appcache CACHE는 fixture와 분리된 기존 앱 소유0700 /tmp cache의 절대 경로와 dev/inode를 manifest 및 generated tool에 고정한다. generated tool은 CACHE/session-cookies/cookie-session*/cookies.txt만 최대1MiB로 읽고 고정 합성 literal 등가만 허용한다. source.cookies 직접 입력과 다른 cache는 거절한다. 모든 parent component를 O_NOFOLLOW|O_DIRECTORY FD로 열고 cache identity, session/owner UID0700, copy UID0600/regular/nlink1, 읽기 전후 copy identity 및 전체 parent 경로 재바인딩 identity를 검증한다. 앱 cache는 평면 fixture 상한에 포함하거나 생성/수정하지 않는다. 자격 증명/일반 사용자 cookie를 출력하거나 허용하지 않는다. 슬롯당 before_B/after_B/tool-started 각1개다. checkpoint는 향후 driver가 실제 B 경계에서 호출하는 기록 API이다. tool이 marker를 바꿀 수 없는 보안 경계라는 주장은 하지 않는다. root가 실제 wire capture 및 product receipt를 별도 대조해야 한다.

## 인수 조건

신규 namespace 생성/충돌 거절, 두 슬롯 marker의 create-only/상한, B 이전 tool marker 부재 조회, 합성 cookie 이외/외부경로/oversize/symlink 거절, source/sentinel 보존을 local selftest로 확인한다. fixture selftest는 actual B gate/reader terminal/OS crash proof가 아니다.

## 검증과 handoff

`python3 scripts/cookie_reader_gate_fixture.py selftest` 실제 exit0 및 Python AST parse 통과. 두 슬롯 before_B/after_B/tool-started 생성, 충돌, 외부 경로/일반 literal/1MiB 초과/symlink 거절, source/sentinel 보존을 확인했다. generated fixture는 평면 12파일/64KiB namespace이며 앱 cache를 같은 root에 섞으면 거절한다. 별도 명시 appcache의 실제 제품 세션 하위경로를 허용하도록 후속 수정했다. root가 generated tool argv와 checkpoint를 향후 native driver에 연결한다. fixture는 자동 cleanup CLI를 제공하지 않으며 selftest가 만든 정확한 TemporaryDirectory만 정리한다.

후속 실제 selftest: 제품과 같은 session-cookies/cookie-session-synthetic/cookies.txt 정상 metadata/download 실행, source 직접/외부/다른 cache/일반 literal/1MiB초과/0644/hardlink/symlink/cache identity 교체 거절을 확인했다. Python AST parse 및 selftest exit0. source SHA `7c364c80d0f32ff501e730037c42b9c2dd25728c09ce13bf6809e68f44414915`로 freeze 인계한다. UID 변경 자체는 다른 사용자 권한이 필요하여 실제 OS 변경 시험을 하지 않았고 코드의 fixed UID 검증만 구현했다. 동시 rename race의 exhaustive 스케줄 시험 및 실제 importer/native B/receipt는 root 후속이다. marker는 합성 cookie 검증 뒤 생성되므로 부재만으로 모든 실패한 tool invocation이 없었다고 판단하지 않는다. 실제 wire/PID proof와 함께 판정한다.

Root 독립 실행 `python3 scripts/cookie_reader_gate_fixture.py selftest` exit0/selftest_passed=true를 재확인했다. [로그](../validation/evidence/cookie-reader-gate-fixture-root/selftest.log). actual B gate tested=false이며 product importer/receipt/native crash 연결은 후속이다.
