# S03: Windows 합성 라이브러리 watcher-only 자동 조정 인수

- 상태: done (자동 변경 화면·DB·원 file ID 확인 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R11, R13
- 담당/수정 범위: metrics_recovery 초안; root 통합·실행·검증 — scripts/windows_library_watch_probe.cs, 이 문서, 전용 evidence
- 선행 작업: root loaded-library 실제 실행, 최신 immutable CPU binary와 bounded private host

## 사용자 결과

파일 목록이 열린 상태에서 외부 합성 추가/삭제/이름 변경을 수동 새로고침이나 탭 재진입 없이 자동으로 반영하는지 실제 Windows GUI에서 검증한다. 현재는 collector staging이며 제품 watcher 실제 PASS가 아니다.

## 계약과 범위

기존 원 loaded-library C#은 수정하지 않고 별도 사본을 만든다. fresh10k3byte 합성 파일의 초기 page1 실제 row를 확인한 뒤 자체 파일만 rename/delete/add한다. 첫 파일을 다른 페이지로 넘기지 않으며 자동 목록 업데이트를 최대20초 actual row polling으로 기다린다. 수동Refresh Invoke와 library 재진입은 모두 없고 결과에도 false로 기록한다. 원90초 worker 상한과 원 HWND/HANDLE/creation/privateDesktop/Job/source SHA/naturalexit0/Job0/input 불변을 유지한다. 사용자 Documents/실제media/visible/globalinput/clipboard는 없다.

원 rename identity는 synthetic 원 파일을 read+shareReadWriteDelete FileStream으로 열고 GetFileInformationByHandleEx(FileIdInfo=18)에서 volume serial+128bit file ID를 얻은 뒤 닫는다. rename 후 동일 방식으로 얻어 원 ID 일치를 비교한다. metadata query 실패는0 또는 추정 PID로 바꾸지 않고 실패다. 이 증거는 native 파일 rename 유지이며 DB reconcile이 원 ID를 사용했는지는 root의 원 DB file_identity 전후 대조가 별도 필요하다. source/root containment은 기존 자체 생성과 no-reparse 경로 검사로 제한한다. file ID가 같다는 이유로 파일 삭제 권한을 얻지 않는다.

## 인수 조건

actual page1 media00000→소유 file rename/add/delete→auto-added/renamed UIA actual row 및 삭제 row 부재와 원 file ID 일치, 단계별 PNG/UIA/원 ID JSON을 확보한다. UIA virtualized visible tree에서 row 부재만으로 DB 전체에서 삭제됐다고 단정하지 않는다. root read-only DB로 present10000/추가·rename·삭제 및 원 file_identity를 확인한다. watcher-only automatic action과 manualRefresh 시험을 섞지 않는다. scanclosewait와 loaded 비용은 별도 범위다.

## 검증과 handoff

Staged source SHA256 `fea070207757f799b9c46b5cbbb93f0ec4a1b3d9e8e76a9d39f441410ebd4ce5`.

실제 수행: 코드 생성 후 정적 검사에서 manualRefresh Invoke가0개, library Invoke가1개임을 확인했다. C# compile/native GUI/Cargo/프로젝트 source 변경은0이다. rootlibrary-ready79274와 동시에 실행하지 않았다. 실제 compiler/native FileIdInfo ABI와 UIA 지원은 root의 다음 fresh 실행에서 확인한다. 원 loaded draft 및 runner는 그대로 보존한다.

## 초기 scan과 watch 등록 관측 구분

원 initial row만으로 초기 scan 완료를 판정하는 약점을 보강했다. 변경 전 원 exact Edit/Apply enabled bounded wait를 수행하고100ms 간격10회에서 원 media00000 row/controls enabled/watch failure 안내 부재를 원시 JSON으로 남긴다. 실패는 외부 변경 전에 거절하며 기존90초 전체상한을 유지한다.

source 순서 확인: accessible_library::run_worker는 WatchSession::new를 command loop 전에 만들지만 실제 WatchSession::new는 watch=None이며 native 등록은 이후 watch.poll에서 이루어진다. command 초기Refresh snapshot 다음 recv_timeout100ms 무command 단계에서 poll된다. 따라서 constructor 존재를 native 등록 완료의 직접 증거로 쓰지 않는다. 현재 UI는 positive subscription ID/등록완료 label을 노출하지 않으므로 `watch_subscription_directly_observed=false`를 기록한다. 안정된1초 UI와 경고 부재는 관측이며 source100ms 순서는 추론이다. 이후 사용자 Refresh/reentry 없이 actual 자동row 변화가 인수지만 nativecallback만이 유일 원인이었다고 증명하거나 background scan in-flight 부재를 UIenabled만으로 확정하지 않는다.

최종 staged SHA256 `879f230c4d4e10dce90f2101a4718d0d45ac25239c9a994c0463693ee5b66a54`. 새 compile/native 실행은0이다. root가 actual 단계별 JSON/PNG와 DB를 독립 검증한다.

## root 실제 native 통합 검증

root가 SHA검증 후 scripts/windows_library_watch_probe.cs를 통합하고 bounded host로 C# compile/native를 실행하여 exit0 PASS했다. windows-library-watch-20261004-resume에서 변경전10sample 안정UI/수동Refresh0/재진입0 후 실제 추가·rename행과 삭제행 부재를 확인했다. root 실제 PNG/read-only DB는 present10k/삭제path부재를 확인했고 원FileIdInfo 전후volume+128bit 동일 및 renamed DB windows:v1 identity가 원nativeID와 정확히 일치했다. root-review.json에 독립 기록했다. 원subscription등록완료 직접관측/scanCloseWait/loaded비용은 미검증이며 전체R03 완료는 아니다.
