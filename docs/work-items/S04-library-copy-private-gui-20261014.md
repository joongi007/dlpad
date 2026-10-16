# S04: 실제 private Library 복사 UI 인수 후보

- 상태: done / 합성 데이터 private GUI 범위
- 분류: test / UI vertical acceptance
- 우선순위: P1
- 요구사항: R02, R08, R11
- 담당/수정 범위: installer_batch — 이 새 candidate namespace의 observer.cs/collect.py 및 guard/handoff; Root — 실행·제품/공유 문서 통합; 현재 Git exit128, 기존 .git 보존
- 선행 작업: Root 검토 whole22 source669/현재 CPU, 실제 v5 public API 10case PASS, Windows private desktop UIA transport

## 사용자 결과

파일 목록에서 선택한 합성 파일을 같은 폴더에 한글 새 이름으로 복사한다. 원본은 보존하고 같은 이름의 기존 대상은 거절한다. 탭 이동 시 실제 표시 상태를 기록한다.

## 계약과 범위

새 fixture만: project-owned windows/tmp/dlpad-library-copy-owned-UUID, data/media/sentinel. 8 MiB 합성 .mp4 bytes를 64 KiB chunk로 생성하고 읽는다. 실제 영상·네트워크·인증·Documents·OS picker는 사용하지 않는다. 현재 CPU를 원본 hash-pinned 복사본으로 실행하며 명시 child environment block에 owned DLPAD_TEST_DATA/LIBRARY_ROOT를 넣는다. 새 detached worker 없음; 원 gallery Job/desktop/MTA/180s worker/230s host/5s natural close 및 실패 cleanup 정책 보존. 현재 제품 파일·Cargo·원 helper·E를 수정하지 않는다.

## 인수 조건

실제 UIA Name/role/RuntimeID/Value와 owned HWND messages로 source 행 선택, 복사 action, 기본 same-parent, 한글 filename readback, 제출, 다른 탭 상태 관측, Library 완료, exact 새 파일 SHA/full128 native ID와 원본 보존, 다시 같은 filename 제출 후 충돌 문구·target 보존. 실제 process creation/HWND identity·WM_CLOSE app0·private Job0·inputDesktop unchanged·원 source/current3/GNU/whole/raw/collector pins beforeafter. 총 six PNG는 Root 직접 픽셀 검토 전까지 미인수. 빠른 완료로 진행/Stop 상태가 관측되지 않을 수 있고 late cancel 인수로 표현하지 않는다.

## 검증과 handoff

Python AST 및 pure evidence guard 6 tests PASS. C# compile/native/UI 실행 0회; 실제 selector/filename focus/완료 toast/default parent 식별은 Root 실행에서 관측해야 한다. caller permission 없이 system desktop/registry/security 변경 없음. 새 proof나 source를 쓰려면 source baseline drift를 먼저 확인하고 fresh candidate로 rebase한다. docs/work-items/S04-library-named-copy-export-20261013.md 연계 초안이며 공유 문서는 Root만 갱신한다.

실제 Root 실행 완료: collect.py 명령(위 frozen HANDOFF의 exact argv), C# compile0/native0/앱 자연0/Job0·private input unchanged. 8 MiB 원본과 복사본 SHA-256 `cad65a1f46032f6d4ecf9086f0093778849d5ece5a6722471600fd02c80ee025`, 서로 다른 full128 Windows ID의 원수신 및 원본 보존을 대조했다. 한글 filename readback, 같은 폴더 복사 완료, Download 탭 footer 실제 완료, 동명 기존 대상 거절을 검증했다. Root source669/39 pins/원 whole·raw와 six PNG 원픽셀을 직접 검토했다. [원자료](../validation/evidence/current22-library-copy-private-ui-v1-20261014/result.json), [독립 검토](../validation/evidence/current22-library-copy-private-ui-v1-20261014/root-review.json), [픽셀 검토](../validation/evidence/current22-library-copy-private-ui-v1-20261014/root-pixel-review.json). 물리 IME/picker/진행 중 취소·CloseWait·실제 codec/외부 영상 인수는 포함하지 않는다. 목록 metadata 두 행의 간격 개선은 S00-library-selection-summary 별도 작업이다.
