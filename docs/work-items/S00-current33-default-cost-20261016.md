# S00: source33 기본 shipping CPU 비용 관측 후보

- 상태: done-observation — 평균 목표 관측, strict CPU17통과/1실패 보존
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08, R11, S00
- 담당/수정 범위: search_reveal; 이 문서와 `.local-tools/session-recovery-20261013/current33-default-cost-prep-20261016/`만
- 선행 작업: actual whole33/rootreview/source675/기본CPU9521b0b999c666f76c75646e0701e41d7d5d48aa062dfd9b85a7462c4ce17ffd 및 CI/native/Cargo/GUI 전부 terminal

## 사용자 결과

최신 기본 shipping CPU를 동일 조건의3×60초/18개 고정10초 interval로 관측할 Root 실행 후보를 제공한다. 기존32 엄격CPU17통과1실패를 새 성공으로 바꾸지 않는다.

## 계약과 범위

current32 default18/composite 및 third-only recovery의 원 measurement-prefix/예산/시간/합성fixture/Job·UIA수명 조건을 유지한다. 회복 observer는 원 소유 HWND/PID/creation의 setup 재조회만 사용한다. 새 source/shared/원32artifact 변경0. candidate 준비는 B만이며 문서는 이 기록으로 동결한다.

## 인수 조건

실제 실행은 Root가 CI 및 모든 경쟁검사 종료 후 조정한다. fresh3process·각10000개3byte 합성파일=30000개·DB3, LoadedReady 원조건/안정화·3×60초·18window·singlecoreCPU<1%와 working/private각80MiB, 원 bounded로그/PNG/Job/사용자입력 불변을 유지한다. 평균값으로 strict interval 실패를 숨기거나 첫 interval을 버리지 않는다. 원 mixedobserver32·quiet_host_verified=false·비원자counter/귀속 제한을 보존한다.

## 검증과 handoff

준비 범위의 성과·정적검사는 B의 HANDOFF/STATIC_RESULT/pins로 인계한다. 이 문서 작성 시점 actual33 관측/native/Cargo/대용량쓰기0이다. 문서는 CI export freeze 전에 동결하며 이후 수정하지 않는다. 실제 관측·30000파일/3DB·원raw 재계산·Root PNG 검수 전 성능 인수 완료를 선언하지 않는다.

## Root 실제 실행 및 결과

전체CI/독립review terminal 뒤 `cost-collector.py WHOLE33 E/current33-default18-budget-20261016 --no-concurrent-dlpad-probes`를 한 번 실행했다. 원검사 Python1/nativeidle2는 strict CPU실패이고 measured3앱은 모두자연0/Job0·18raw완전이다. 별도 `review.py --measurement-terminal-confirmed`와 `run-native.py --measurement-terminal-confirmed`는 실제0으로 종료해 원실패를성공승격하지않고 관측/WinNative30,000파일·3readonlyDB를 확인했다. Root6PNG도 직접확인했다. 평균0.5723777454%/0.4941536640%/0.4941599523%,최대작업집합77.125MiB,strictCPU17PASS/1FAIL1.0920442160%·memory18PASS다. [Root 검토](../validation/evidence/current33-default18-native-root-review-20261016/root-review.json).

Root 최종산출물 집계의 첫읽기는 native pins를 object로 잘못가정해 reviewer assertion으로만 실패했다. 실제 pin은 SHA문자열이므로 원형에맞게대조한후0으로 완료했다. 원측정/기준/원파일은변경하지않았다. 물리/quiet host·원인귀속/전체P0 미인수는 유지한다.
