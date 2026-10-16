<!-- Historical v14 snapshot; see current README for accepted candidate. -->
# 앱·설치 UI/UX 검수 — 2026-10-13

현재 v14 whole은 **1,305 성공 / 0 실패 / 30 ignored**이며 source656·43harness·현재3실행 파일을 독립 대조했다. 배율 변경 뒤 최초 늦은 viewport 축소를 기존8frame 공개로 보정하며 사용자의 wheel/press/key/text/IME·페이지 이동은 감시를 해제한다. 실제 Candidate 회귀5개와 격리63화면 검수, 최신 archive·변조 실행 거부·독립27ID가 통과했다. 앱은1,536B 증가하고 설치기·CLI·Cargo·의존성은 그대로다. 설치·실행 내부 native0/Job0와 외부143 종료는 분리하며 외부 원인은 미확인이다. v13 CI·비용·긴 경로 결과는 당시 소스에 한정하며 v14로 소급하지 않는다. 물리 pointer/IME/reader/OS DPI·전체 UI/UX 인수·정식 배포는 미완료다.

[현재 전체 검사](evidence/windows-msvc-scale-viewport-reflow-whole-20261013/root-review.json), [현재63화면](evidence/current-scale-viewport-reflow-app63-20261013/root-team-visual-review.json), [현재 배율 회귀](evidence/windows-msvc-late-footer-scale-green-v3-20261013/root-review.json).

아래 v13 항목은 이전 검증 snapshot이다. 배율 일시 가림의 현재 수정 상태는 위 v14 기록을 따른다.


Rust 네이티브 Windows 앱과 설치기를 합성 데이터·격리 데스크톱에서 검증한다. 사용자 화면을 강제로 활성화하지 않는다. 전체 물리 UI/UX 및 정식 릴리스 인수는 아직 미완료다.

| 항목 | 실제 확인 | 남은 범위 |
|---|---|---|
| 최종 소스 | v13 fmt/clippy/test/release1,297/0/30·43harness·source656·원3artifact 독립 대조 | GUI/CI는 각각 별도 검증 |
| 공통 선택·호버 글자 | Light 대비 RED를 실제 paint에서 재현, 수정 후 두 테마4상태 회귀 PASS | 물리 포인터와 OS 전달 실패 조사 |
| 앱 화면·폰트·배치 | v13 63PNG 수집·소스·3DB·자연 종료 검증과 팀 전체 직접 시각 검수 | footer 줄바꿈 뒤 배율 조작 일시 가림, 물리 입력과 사용자 평가 |
| 입력 오류 | 0093 CPU의 Dark200/Light100 전체문맥·거절/수정/끄기/자동 복귀 PASS | 새 빌드 전체 입력을 소급 인수하지 않음 |
| 파일 행 | 0093 CPU 단일 합성파일 선택·원ID·경로·닫기복귀 및4PNG PASS | 최신 Dark/확대/10k/바쁜행 |
| 설치 후 실행 | v13 완료 Light100/Dark200·경로 변경·재검증·실행·원감독 생존·양쪽 자연0/Job0, 변조 거부·자식 미생성·자연1 PASS | GUI-ready·늦은 실패의 사용자 안내와 물리 환경 |
| 최신 설치 ZIP | v13 실제 archive/source/3binary/graph258·선택 notice 누락0 PASS | 실제 설치 UI와 긴 경로 별도 |
| 긴 경로 설치 | v13 577/598UTF16 설치·Verify·완료 안내와 독립27ID PASS | 긴 경로 앱 실행·clean OS·upgrade/rollback |
| 테마·창 제목 | f15 본문/선택/Get20 0/1/0 PASS | Get35/36 오류·물리 DWM 색상 미검증 |
| 기본 비용 | v13 단독3×60초 reported 평균0.546/0.599/0.599%, 메모리18PASS; strict CPU17PASS/1FAIL(1.094%) | 원FAIL 유지·정확한 인과/기준 환경; 감독 프로세스 별도 비용 |
| 로컬 CI | v13 실제15단계·7captures·8Cargo 및1,954 export행/1,955 config파일 독립 대조 PASS | hosted CI·최종 배포; 문서는 export 당시 snapshot |
| 릴리스·Git·updater | 개발 설치 후보,원.git 보존 | 정식 서명·공지·물리환경·Git/hosted CI·updater |

[최신 설치 ZIP](evidence/dlpad-install-ui-cross-surface-v14-20261013.zip), [최신 단독 실행](evidence/ui-test-scale-viewport-reflow-20261013/dlpad.exe), [남은 작업](P0-remaining-20261011.md), [검수 작업](../work-items/S00-ui-cross-surface-audit-20261013.md). 이전 결과는 그 소스·실행 파일에 한정한다. S10 P0 진단은 최초 릴리스에 포함한다.

63개 이미지의 경로·SHA를 독립 대조했다. 한글 누락·겹침은 검수 이미지에서 발견하지 않았다. 원 확대 이미지 두 개를 root가 다시 직접 확인하니 다섯 메뉴가 모두 보였다. 기존 ‘두 메뉴’ 메모는 오관찰로 정정하며 원보고·이미지를 보존한다. 200%에서 앞선 unfocused 테마 라벨이 scroll 밖에 있는 상태는 정상적인 이전 행 일부다. 별도 native 초기 진단의 footer 두 줄 표본에서 활성 배율 상자 아래가 일시 잘리는 실제 발견 사항은 회귀·수정 후보를 준비 중이다. [전체 이미지 검수](evidence/current-completion-copy-app63-20261013/root-team-visual-review.json), [현재 로컬 CI](evidence/current-completion-copy-local-ci7-20261013/root-review.json), [긴 경로 ID](evidence/windows-msvc-ui-cross-surface-v13-20261013-long-install-independent/root-review.json). IsOffscreen 미지원 값을 실제 가림으로 해석하지 않는다.

[메뉴 관찰 정정](evidence/current-completion-copy-app63-20261013/root-visual-correction-review.json), [실제 일시 가림](evidence/current-completion-copy-zoom-navigation-diagnostic-20261013/root-transient-scale-visual-finding.json), [후속 작업](../work-items/S00-scale-viewport-reflow-20261013.md).

[현재 비용 독립 검토](evidence/current-completion-copy-default18-cost-20261013/root-review.json)는18구간 raw 수학과 readonly3DB/3만 fixture를 확인했다. reported60초 평균의 마지막 raw accounting counter는 producer에 없어6구간 weighted mean과 구분한다. 단위 회귀와 물리 UI는 별도이다.
