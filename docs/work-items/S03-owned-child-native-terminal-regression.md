# S03: OwnedChild RAII 이후 실제 사용자 서비스 terminal22 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서와 전용 신규 evidence, 명시 소유 immutable /tmp helper/driver snapshot
- 선행 작업: root OwnedChildLifetime RAII sourcefreeze/전체7 검증 및 최신 helper·typed driver 빌드 READY

## 사용자 결과

작업 owner의 오류·종료에서도 소유 launcher 수명을 회수하며, 실제 서비스 terminal 확인이 있는 종료와 확인 누락을 구별하는지 합성 시험으로 검증한다.

## 계약과 범위

README → PRODUCT → CONTEXT, R08/R10/R11, ARCHITECTURE, ADR-0002와 기존 terminal/OwnedChild 작업을 기준으로 한다. 기존 scripts/terminal_user_service_probe.py와 terminal_user_service.rs 계약은 읽기 전용으로 사용한다. 제품/Cargo/source와 기존 서비스/8h soak를 변경하지 않는다. 빌드 확인 전 stale binary 실행은 금지한다. 새 임시 binary 복사는 SHA before/after와 adjacent helper를 고정하고 source binary를 덮어쓰지 않는다. Root가 source/Cargo 통합 검증을 담당한다.

## 인수 조건

- 실제 API17: exit0/42/126, metadata, cancel/timeout, owner/guardian/both SIGKILL, manager 누락/unknown backend, FD12반복, bounded backpressure/outputlimit, startup3fault.
- raw3: 빠른 exit0/42/126의 socket frame 정확히 RGCLC, pipe/socket EOF, 동일 launcher PID 소실 및 pidfd ready.
- fake2: GC 또는 LC 누락은 typed cleanup_unconfirmed, 도구 미기동, 소유 helper 소실, FD 불변.
- 모든 관측은 고유 unit/InvocationID/MainPID/start/cgroup으로 소유권을 확인하고 owned PID/pidfd만 정리한다. 기존 unit을 대상으로 삼지 않는다.
- 무관 소유 sentinel 생존·temp 정리·driver/helper digest 불변·raw 결과 보존. Linux user-manager 실증으로만 보고한다.

## 검증과 handoff

Runner/driver 읽기와 범위 기록 완료. 신규 binary 빌드 READY 전 실제22case 실행은 하지 않았다. 원시 실패를 보존하고 확인되지 않은 terminal/PID/cgroup 정리를 PASS로 쓰지 않는다.

Root가 이번 source의 default workspace test exit0와 최신 helper/driver 빌드를 확인한 뒤 실행했다. Source→copy before/after SHA가 일치한 새 immutable `/tmp/dlpad-owned-child-terminal-binaries-gap6fjqj`를 사용했다. 실제 명령:

```sh
python3 -B scripts/terminal_user_service_probe.py /tmp/dlpad-owned-child-terminal-binaries-gap6fjqj/typed-user-service-driver /mnt/d/project/dlpad/docs/validation/evidence/owned-child-terminal-native-20261004
```

exit0, 실제22case 모두 PASS. [전체 원시](../validation/evidence/owned-child-terminal-native-20261004/result.json), [명령](../validation/evidence/owned-child-terminal-native-20261004/command.txt), [immutable 입력](../validation/evidence/owned-child-terminal-native-20261004/immutable-snapshot.json), [추가 읽기 전용 재대조](../validation/evidence/owned-child-terminal-native-20261004/post-verification.json)를 보존했다. 8개 제품/driver/harness source는 launch observation SHA와 동일한 bytes snapshot을 따로 보존했다.

API17case의73 identity와14 cgroup/unit에서 exact PID/start 부재·pidfd ready·cgroup 제거·raw terminal LoadState=not-found를 확인했다. raw3에서 실제 socket `RGCLC`와 exit0/42/126·pipe/socket EOF·정확한 launcher 소실을 확인했다. fake2는 GC/LC 누락 시 typed cleanup_unconfirmed·tool 미기동·정확 helper 소실이며 실제 service 인수와 구분한다. 실행 뒤 추가 재대조한 총78 identity와14 cgroup도 모두 부재였다. 모든 실행 결과의 FD before/after는4→4, unrelated sentinel 보존, owned temp 제거, binary digest 불변이었다. manager 누락/guardian 소실/startup3의 typed 오류와 진짜 tool126을 혼동하지 않았다.

Driver SHA256 `4b6352b5461a4238bb707f26e6491fd6a7b20bc0bf3556e5829e9e63cb7c9503` (40,608,552 bytes), helper `8e7a370d91eea794eed5674598212fdec8c31448bfdc3320c7923ff692957d8b` (37,732,152 bytes), 직접 확인한 process.rs source `72bb3c48a756f44e14290f35acf1d56ffe23fa48472d43eb04c61ef2937c3d02`를 기록한다. 테스트 전용 debug binary 크기이며 제품 release 크기가 아니다. Root 이후 attachErr alias 검토/수정과 현재 snapshot 인수는 구분한다.

이 작업의 done은 Linux debug immutable22case 회귀 완료다. 전체7 Cargo pipeline 완료/최신 GUI/Windows·macOS/실계정·네트워크·8h·최종스택/P0 전체 완료를 뜻하지 않는다. Runner/test 제품 소스/Cargo를 수정하지 않았다. Root가 원시와 제품 전체 검증을 독립 통합한다.
