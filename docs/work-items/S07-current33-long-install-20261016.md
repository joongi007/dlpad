# S07: current33 긴 경로 설치·검증·실행 smoke

- 상태: done-scoped — 실제 긴 경로 설치·재검증·설치 앱 수명 및 Root 검토 통과
- 분류: validation
- 우선순위: P0
- 요구사항: R08/R11, ROADMAP S06/S07
- 담당/수정 범위: field_reveal. 새 scratch current33-long-install-prep-20261016 및 이 문서만. 기존32 helper/evidence·제품·shared scripts·Cargo/lock 변경 없음.
- 선행 작업: actual whole33/Rootreview/current3PE 및 generated33 payload. Root 단독 실행·CI export source freeze 조정.

## 사용자 결과

현재 v33 설치기를 새 소유 한글 긴 경로에 설치해 다시 검증하고 그 설치 경로의 실제 앱이 열리고 자연 종료하는지 확인한다. 일반 업그레이드·사용자 기존 설치 덮어쓰기 시험이 아니다.

## 계약과 범위

기존 current32 long collector의 원3PE/whole/manifest/binding·전체payload5·retainedHANDLE/NativeID·Job/데스크톱 계약을 재사용한다. 새 root+한글80자 segment5개·bundle/exe577/598UTF16 이상, data/media는 별도 짧은 소유 경로다. 외부 도구를 선택하거나 다운로드하지 않는다. 모든 자동 GUI는 private desktop이며 사용자 문서/미디어/clipboard/global입력/visible fallback 없음.

별도 checkbox/gallery와 popup24장 호출은 이번 smoke에서 생략하며 그 성공을 주장하지 않는다. readonly startup-hint/minimum policy·도구 미선택을 유지한다. stage-copy·readonly journal·설치/검증·원앱 수명에 결합된 원 검사/캡처는 보존한다. source/record/active schema·원 사용자 데이터 변경 없음.

## 인수 조건

실제 current whole33/Rootreview/source/3PE와 generated7파일 원SHA, 새 installed5entry·20objects/6parents NativeID/SHA·원 journal5기록 동일, create-only 설치/readonly verify·원target수명·설치기/앱 자연0/Job0·private desktop/원 input desktop 불변. 정확 긴 path 길이/원실행 image identity를 확인하고 원PNG는 Root가 직접 읽는다. static 준비를 실행 성공으로 기록하지 않는다. 전체 UI/물리IME/reader/cleanOS/서명/일반upgrade 완료로 확대하지 않는다.

## 검증과 handoff

actual whole33 Rootreview의1670PASS/0FAIL/42ignored/43harness/source675와 generated33 manifest/binding을 읽었다. 준비 단계 Python AST/원 helper 대조만 수행하며 native/Cargo 실행0. exact argv·고정핀·gallery 제외 경계는 scratch HANDOFF.md에 있다. Root가 CI export freeze와 충돌하지 않는 시점에 단독 실행하고 actual evidence/결과를 이후 기록한다.

Root가 고정7candidate pin을 대조하고 argv.json 그대로 실행했다. actual compile/native terminal0, private desktop/input 불변·설치기/앱 자연0/Job0이다. Root 독립 reviewer도 source675·whole33·원46pins·577/598UTF16·20objects/6parents·journal5·installedpayload5 원bytes/SHA 및 실제 child image identity를 재검증해0으로 종료했다. Root가 dark200 top/bottom·installed앱3PNG를 직접 읽고14PNG 전체 SHA를 고정했다. [원 검토](../validation/evidence/windows-msvc-ui-feedback-v33-long-install-private-v1-20261016/root-review.json). 별도 팀14PNG 전수 검토는 이 시점 대기이며 full44갤러리/물리입력/전체P0를 주장하지 않는다.

별도 팀검수도 후속 완료했다. 원14PNG를 모두 직접열고101개receipt/원파일 조건을 대조했으며 Root는원보고 SHA와자신의원46핀/3PNG검토를 구분해보존했다. 전체44갤러리/physical hover를 새로인수한 것은 아니다.
