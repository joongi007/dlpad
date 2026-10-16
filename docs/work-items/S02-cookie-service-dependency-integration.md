# S02: gated service와 auxiliary parent 의존성 통합

- 상태: ready (소스 freeze, 통합 실제 검증 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; process/user_service.rs 및 이 문서
- 선행 작업: root 독립 dependency3case/failure2case native proof

## 사용자 결과와 계약

gated service와 StopWhenUnneeded=true auxiliary parent를 같은 StartTransientUnit transaction에 생성한다. gated-only RemainAfterExit=true/ignore-failure로 native empty와 receipt 확정 수명을 확보하며 원 ExecMainCode/Status를 유지한다. normal/Drop parent 직접 Stop을 제거하고 exact reader service Stop/Unref 뒤 parent Unref만 사용한다. nongated 경로 최소 변경. 새로운 schema/CLI/Cargo 변경 없음.

## 인수 조건

active/exited/MainPID0 gated terminal 지원에 pidfd 원 guardian 종료 필수. held/fresh parent strict native empty 및 receipt sync 유지. live/queued foreign sibling 직접 Stop0. actual success/failure/signal/cancel/crash native 회귀는 root 실행 담당이며 소스 작성으로 완료 주장하지 않는다.

## 검증과 handoff

실제 변경: start의 sliceSome gated 경로만 auxiliary parent AddRef/SWUtrue를 service와 단일 transaction에 담고 RemainAfterExit=true 및 ExecStartEx ignore-failure를 적용했다. parent 선행 별도 start는 없앴으며 accepted transaction 뒤 service/parent Invocation을 capture한다. wait_unit_seeded는 gated bool로 active/exited/MainPID0을 terminal로 받아들이되 기존 seeded guardian pidfd가 종료됐어야 한다. 원 ExecMainCode/Status 변환과 service cgroup cleanup, parent native strict empty 및 durable receipt write와 전후 identity 검증을 유지했다.

receipt 이후 exact reader service Stop→wait_cleanup→Unref, parent는 sameInvocation Unref만 수행한다. ReaderContainer normal/Drop의 직접 Stop을 제거했다. Drop은 service만 소유 cleanup 권한을 사용한다. parent는 살아 있는 또는 queued dependency가 있으면 manager가 보존해야 하며 이를 동기적으로 강제 Stop하거나 전부 없어지도록 기다리지 않는다. source capture 이전 identity를 얻지 못한 resourcefail에서는 unit 이름만으로 추정 Stop하지 않는다. 외부 sibling의 namespace 경쟁을 원자적 권한 경계로 주장하지 않는다.

단독 owned rustfmt 실제 PASS. frozen SHA256 `2d44325f79ea678a1924bd62a2b69fa4f5f0d0bff4a4fc0c54b400085880ae49`. 담당 에이전트 Cargo/native 실행 없음. root가 helper 재빌드 뒤 cookie_service_gate, cookie_download_gate, cookie_recovery_gate 및 nongated/process 전체 회귀를 실행해야 한다. 실제 제품 foreign queued/sibling 및 resourcefail cleanup 인수도 별도로 필요하며 이 소스 작성으로 P0 완료를 주장하지 않는다.

## 최초 실제 통합 실패와 진단 보강

root cookie-priorboot-dependency-second/service-native.log 실제 native3개 모두 실패했다. I 수신 직후 fixture service InvocationID가 비어 있었고 launcher는 reader container invocation changed로 종료했다. helper build0 및 lib251PASS는 이 native 실패를 대체하지 않는다. 부모 SWU 또는 guardian 제어 경로가 원인이라고 아직 확정하지 않았다.

group()의 실제 Invocation 불일치와 ActiveState 비활성을 별도 고정 오류로 분리했다. debug 빌드에서만 exact env DLPAD_TEST_CLEANUP_STAGE_TRACE=1을 허용해 명시 wait_unit_seeded 실패 원인별 고정 stage 및 gated wait-error/terminal/stop-error/cleanup-error의 ErrorKind/raw errno를 출력한다. 경로/unit/argv/자격증명은 출력하지 않는다. 기존 timeout/proof/공유 control 계약은 완화하지 않았다.

소유 rustfmt PASS, 새 frozen source SHA256 `0f166558868c5a2902eac7804128edfb4d1ae63837acef4dadddc1e22b4c8548`. 자체 Cargo/native 미실행. root 최신 helper rebuild 후 opted-in native 원시 대조가 필요하다.

최종 root 재검증 `cookie-dependency-current-native-eighth/result.json`은 최신 source manifest와 helper SHA를 보존하며 source_unchanged=true, debug trace enabled 상태에서 service3PASS/download4PASS/recovery2PASS다. `cookie-priorboot-dependency-workspace-seventh/result.json`의 전체 fmt/Clippy/test/release도 모두 exit0이며 원시 소스 해시가 일치한다. CPU feature/Windows와 initial unknown identity 및 제품 foreign queued 경계는 이 통과로 대신하지 않는다.

## 최신 root native 원시 독립 대조

root가 SubState 조회 interface를 Service에서 Unit으로 바로잡았다. 이전3FAIL의 wait-unit-error errno53 원시는 유지한다. 이후 service3PASS/download4PASS를 확인했으나 debug trace ON killed recovery는 deadline10초 FAIL, 같은 binary trace OFF recovery2PASS였다. root는 닫힌 stderr에 eprintln이 panic할 수 있는 경로를 best-effort writeln(stderr) 오류무시로 바꿨다. trace ON sixth recovery2PASS가 후속 근거다.

stress_recovery는 최신 `evidence/cookie-dependency-current-native-eighth/result.json`과 service/download/recovery.log를 직접 읽었다. source_unchanged=true/trace_enabled=true 및3명령 exit0이며 실제 service3PASS7.80초, download4PASS9.03초, recovery2PASS4.78초를 확인했다. service 로그는 wrongB에서 begin unconfirmed를 유지하며 failure를 execution success로 승격하지 않는다. killed recovery 첫 요약0/1/1, success1/0/1이며 외부 sentinel 기준 rejected1 계약을 지킨다. 읽기 시 user_service source SHA256 `b0d3b7c2a058eb918a39e6408499914cfa252ae8c453b2a071f02d7167ae5499`로 root 수정 snapshot을 구분한다. 직접 Cargo/native 재실행은 하지 않았다.

이 scoped3+4+2 PASS로 초기 identity 미확보 resourcefail 잔여물, 실제 제품 foreign queued fixture, 실제 reboot 및 간헐 output-limit 원인을 완료로 표현하지 않는다. keeper 대안 실험은 별도 작업이며 현재 RAE 제품 정책 변경을 자동 승인한 것이 아니다.
