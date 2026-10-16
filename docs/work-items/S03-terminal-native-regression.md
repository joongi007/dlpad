# S03: native terminal ownership 실제 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, R11, R13
- 담당/수정 범위: metrics_recovery — scripts/terminal_user_service_probe.py, crates/engine/tests/terminal_user_service.rs, 이 문서
- 선행 작업: root terminal productsource/Cargo/helper READY; stress parser

## 사용자 결과

동일 immutable helper에서 실제 GC/LC와 tool exit0/42/126, API cancel/timeout·guardian death·manager 부재를 검증한다. 이전 strict13 기대는 새 typed 결과로 명시하며 실패를 성공으로 완화하지 않는다.

## 계약과 범위

기존 user_service_engine_probe execute의 ownpidfd/InvocationID/cgroup/cleanup 증거를 읽기 전용으로 재사용한다. 새 driver는 typed unconfirmed field를 기록한다. raw socket fixture는 합성 빠른 tool만 실행하고 GC+LC를 직접 읽는다. 사용자 desktop/실미디어/credential/기존unit 신호는 없다. 새 file만 수정하고 Cargo는 root가 담당한다. 빠른 fixture에서 시작cgroup 관측 실패는 미인수이며 원시 실패를 보존한다.

## 인수 조건

realnative strict13+startup 실패, genuine126과 rawfast0/42/126, exact child/cgroup cleanup/sentinel/FD/pipeEOF, binary hash 전후를 확인한다. missing G/L sourcefixture는 manager-independent scoped fake helper로 따로 구분한다. stdout에 fake control 문구를 쓰는 것으로 terminal 확인을 대신하지 않는다.

## 검증과 handoff

아직 새 시험 미실행. root READY binary를 받은 뒤 실행하며 source/digest/명령/cleanup 결과를 기록한다.


## 실제 native 결과

명령: `python3 scripts/terminal_user_service_probe.py /tmp/dlpad-terminal-native-kg13jr0t/typed-user-service-driver /mnt/d/project/dlpad/docs/validation/evidence/terminal-native-first-20261004` — 실제 exit0,22case PASS. [전체 원시](../validation/evidence/terminal-native-first-20261004/result.json), [실행 명령](../validation/evidence/terminal-native-first-20261004/command.txt), [시험 후 현재 입력 해시](../validation/evidence/terminal-native-first-20261004/source-inputs-after.json)를 보존했다. 제품 입력 해시는 after-observation이며 root의 immutable build 입력 대조와 구분한다.

API17case는 기존 strict13, genuine exit126, startup fault3를 포함한다. 실제 기록된73API identity와14cgroup에서 소유 프로세스 부재·pidfd ready·cgroup 제거·unitGC를 확인했다. guardian SIGKILL은 실제 tool/child/leaf와 InvocationID를 확보한 뒤 정확한 guardian pidfd만 종료했고, API는 cleanup_unconfirmed=true/terminal ownership confirmation missing을 반환했다. missing manager는 tool 미시작과 typed startup-unconfirmed, startup3는 real started unit을 관측한 뒤 typed terminal-unconfirmed로 끝났다. 이전 원시와 기대는 수정하지 않았으며 새 wrapper의 변경 기대를 명시했다.

빠른 합성 Python exit0/42/126은 소유 launcher socket에서 실제 raw `RGCLC`5byte를 읽었다. 각 원 exitcode가0/42/126으로 유지되었고 stdout/stderr pipe EOF, socket EOF, 원 launcher identity 부재/pidfd ready를 확인했다. stderr0byte였다. 실제 raw GC+LC 확인이며 stderr 또는 exit126 문자열 추론이 아니다. 이 빠른 fixture는 descendants를 생성하지 않으며 별도 API17의 real escaped descendant/cgroup 증거로 전체 범위를 나눈다.

missing guardian/launcher acknowledgment2case는 임시 폴더에 immutable driver만 복사하고 adjacent fake owned helper로 RLC/RGC를 보내는 manager-independent sourcefixture다. 둘 다 typed cleanup-unconfirmed, FD동일, 정확 helper identity 부재, tool 미시작이었다. 실제 service cgroup 검증으로 확대하지 않는다. 제품 release 환경 spoof/hook을 추가하지 않았다.

binary SHA256 `79f7b1f35d37e216dcdb7f47a8ba2786273f7c543942ad80f4784f897f498b0e`, helper `90238f822b75a7bac4cab18d8a29507c29e2afd45c8d5693c9f4a2c6bd2c9c8a`는 시험 전후 동일했다. sentinel은 살아있는 동일 identity로 보존했고 자체 terminate/wait했다. owned temp 폴더 제거true. 다른unit/프로세스를 신호하지 않았다. harness/driver와 재사용 script 원본을 증거 폴더에 복사했다.

## 남은 한계와 통합 인수

이 결과는 현재 Linux 사용자 manager 및 immutable debug helper snapshot22case의 실제 회귀다. 다른 OS, 실제 account/media/network, release에서 debug hook 무시, 장기8h, latest native GUI cookie 기능 전체를 검증하지 않는다. noR의 기존 고정 readiness10초와 cleanup5+2 정책, 전체 deadline/cancel startup 반영은 별도 잔여 작업이다. root가 원시 결과/source/binary와 제품 전체 Cargo/GUI를 독립 통합 검증해야 한다. 이 문서의 done은 본 native 회귀 작업 완료이며 P0 전체 또는 finalstack 확정이 아니다.
