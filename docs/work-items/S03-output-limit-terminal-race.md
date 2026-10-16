# S03: 출력 상한 종료 확인 간헐 실패

- 상태: done
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R04, R11
- 담당/수정 범위: root — process.rs, process/guardian.rs, process/terminal.rs 및 전용 회귀·이 문서
- 선행 작업: 시작 deadline 통합, terminal acknowledgement

## 사용자 결과

출력 상한 초과 중 종료 명령과 helper 자연 종료가 겹쳐도 정확 소유 종료 확인을 잃지 않아야 한다. 미확인 오류를 일반 성공으로 바꾸지 않는다.

## 계약과 범위

전체 engine 시험에서 oversized output 첫 단계에 TerminalUnconfirmed가 관측됐다. 단독 exact 시험은 통과했다. typed 보호 정책은 유지하며 원인 재현·관측 후 수정한다. terminal protocol·소유 PID 보호·쿠키 미확인 보존을 약화하지 않는다. Cargo/schema 변경 없음.

## 인수 조건

간헐 경로 재현 또는 통제된 race fixture로 socket frame·helper 종료·signal ordering의 원인을 확인한다. 정리 확인과 오류 분류 실패 회귀 및 전체 engine 시험을 통과한다.

## 검증과 handoff

전체시험 197 passed/1 failed/2 ignored exit101 원시 기록은 docs/validation/evidence/startup-checkpoint-linux-full-retry/에 있다. exact 단독 재시험은 1 passed exit0이며 해결 근거가 아니다. 현재 원인 미확정이다.

## 최소 재현과 수정 후보

실제 Unix socketpair에서 GC를 보낸 상대가 unread K를 남긴 채 닫으면 앱은 GC 뒤 EOF 대신 ECONNRESET104를 받았다. read/write/both shutdown만으로 reset은 없어지지 않았다. 이 모델은 실제 실패 원인 후보이며 제품 원인 확정으로 확대하지 않는다.

root는 Linux의 이미 reap된 guardian/launcher에 poststatus K를 보내는 경로를 제거했다. 취소 grace에서 reap된 경우에도 K를 생략하고, 마지막 중단 명령 뒤 앱 control의 write 방향만 shutdown하여 읽기 GC/LC/EOF 검증을 유지한다. guardian endpoint 또는 launcher의 공유 write 방향은 닫지 않는다. 전체 engine 회귀가 진행 중이며 해결 완료로 표시하지 않는다. 다른 OS의 group/Job cleanup은 유지한다.

## 최신 helper 명시 빌드 뒤 실제 회귀

root가 guardian helper를 명시 재빌드한 뒤 출력 상한/metadata 상한/취소 회귀가 통과했다. 첫 전체시험에서 owned child procfs read ESRCH 경합1개를 보존하고 해당 소멸 errno만 정확 처리했다. 재시험은 전체 engine exit0이며 [독립 검토](../validation/evidence/guardian-reader-drain-integration/root-review.json)가 핵심6case와 소스/helper 해시를 기록한다. 실제 service cgroup·단기 반복 race·전체 workspace와 다른OS 인수는 별도다. 이전 실패를 삭제하거나 통과로 바꾸지 않았다.

## 종료 수정 인수 결과

Linux guardian control reader가 queued 명령을 소진한 뒤 유한 timeout으로 종료하도록 관리했고, app의 이미 reap된 helper에 보내는 K를 제거했다. 전체 engine library201개 및 실제 service terminal22case가 통과했다. 실제 socket reset은 여전히 typed unconfirmed로 거절한다. 검증 범위는 Linux 종료 수정이며 전체 workspace/다른OS/P0 완료는 별도 게이트다. 원시 증거는 guardian-reader-drain-integration 및 reader-drain-native-terminal-first에 보존했다.
