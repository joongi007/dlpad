# S11: 최신 UI 설치 후보 통합 검증

- 상태: done
- 분류: quality
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: 주 담당 README.md, docs/validation/P0-status.md, docs/validation/P0-remaining-20261011.md 및 validation/evidence의 새 설치 증거. 기존 S11-install-zip-20261011 계약의 후속이다.
- 선행 작업: Windows corrected whole 1143/0/30, 149화면 raw/provider 검토, Dark200 도구3종 scoped 픽셀·SQLite 검토

## 사용자 결과

README에서 현재 UI 수정과 일치하는 설치 ZIP을 받는다. 새 폴더의 실제 5파일 설치와 재검증을 확인했다.

## 계약과 범위

프로젝트 소유 임시 폴더와 private desktop만 사용했다. 기존 payload 정확 namespace, 원본 SHA, readonly journal 및 native identity 조건을 유지했다. 서명·clean OS·현대적 설치 화면 디자인·updater·전체 P0 완료는 이 작업의 완료 범위가 아니다.

## 인수 조건

현재 whole/source와 묶인 ZIP, 258개 selected graph notice, 실제5파일 설치·재검증, journal 전체 문자열·readonly·선택·원본 보존, 자연 app0/Job0, 현재27항목 native ID 대조 및 root 캡처 검토.

## 검증과 handoff

`/tmp/dlpad-current-ui-tool-action-final-package/pipeline.py` graph/package/review 각 종료0. 순수 보호 테스트10개 통과. ZIP 16,493,356B, SHA256 `4a4e5e37726de7b7fdea1c5e96948cfa494307b1f07d5a3616e0e4d98bd3b64c`.

첫 install 사전 검사는 root가 자체 검토 JSON을 payload 안에 둬 실패했다. 그 root 소유 JSON만 증거 경로로 이동했다. 다음 실행은 과거 ValuePattern-only 관측기 회귀로 실패했으며 원 실패를 보존했다. 이전 실제 PASS의 TextPattern/ValuePattern 전체 문자열·Win32 원문·HWND/PID/TID 대조를 복구한 `/tmp/dlpad-installer-journal-provider-diagnosis/install_collect.py` v2는 종료0. 초기/647자 줄바꿈/복원 세 단계 모두 실제 TextPattern 전체 읽기 일치, readonly·48px·전체 선택 확인.

독립 `review_install.py`와 `run_root.py` 종료0. [설치 원본 검토](../validation/evidence/windows-msvc-current-ui-tool-action-final-v2-20261012-install/root-independent-installed-review.json), [root 화면 검토](../validation/evidence/windows-msvc-current-ui-tool-action-final-v2-20261012-install/root-installer-pixel-review.json), [현재 native IDs](../validation/evidence/windows-msvc-current-ui-tool-action-final-v2-20261012-install-native-identities/result.json). 현재 파일 ID는 과거 소유권이나 삭제 권한을 증명하지 않는다. 물리 caret/clipboard·OS DPI·서명·전체 법적공지·최종 사용자 UI 인수는 미검증이다. 설치 화면의 전통적인 Win32 스타일과 넓은 빈 공간은 다음 디자인 개선에 남긴다.
