# S11: 설치 화면 가독성과 기본 크기

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R05, R08, R11
- 담당/수정 범위: 주 담당 crates/app/src/installer_main.rs, 이 문서 및 새 validation/evidence. 분리된 release_preparation 담당은 /tmp 코드·관측기 후보만 작성했다.
- 선행 작업: 최신 실제5파일 설치·재검증과 root 설치 PNG 검토

## 사용자 결과

한글 안내와 선택 항목을 읽기 쉬운 글자 크기로 표시하고, 기본 설치 화면의 불필요한 빈 공간을 줄인다. 기본·최소 크기에서 모든 표시 컨트롤이 창 안에 들어간다.

## 계약과 범위

Native Win32 Rust UI·OS 메시지 글꼴을 유지한다. 작은 글꼴에만 16px 하한을 적용하고 더 큰 접근성 설정은 유지한다. packaged 기본800×620·최소800×600 후보를 검토했다. 입력32px·checkbox28px·journal104 readonly multiline48px와 기존 IDs, 설치 worker·argv·권한·원본 보호 조건을 유지한다. 새 의존성 없음. 실제 DPI/hover/focus·최종 현대적 디자인 인수는 별도로 확인한다.

## 인수 조건

Windows 전체 fmt·clippy·test·release, 기본·최소 실제 HWND client/모든 표시 child 전체 bounds·PID/TID/ID, 세 도구 한글 caption 직접 PNG 검토. 기존 긴 journal 전체 읽기·선택, 실제5파일 설치·재검증·현재27 native IDs를 재검증한다. 비용·크기 변화는 실제 빌드 결과로 기록한다.

## 검증과 handoff

/tmp/dlpad-installer-readable-compact-ui 후보 rustfmt 통과. 첫 580px 높이 후보에서 결과 영역 잘림 가능성을 root가 찾아620px 기본/600px 최소로 수정했다. 원래 실제 PASS 관측기를 바탕으로 /tmp/dlpad-installer-default-min-native의 기본·최소 C# 관측기를 구현했다. 후보 Rustfmt/Python syntax는 실제 native 검사 통과를 뜻하지 않는다. 아직 제품 통합 전이며 원래 설치 ZIP을 유지한다.

주 담당이 원본 SHA를 대조하고 제품 코드를 통합했다. 기본800×620·최소800×600, OS 글꼴16px 하한과 안내 간격을 적용했다. Windows 전체검사와 새 기본·최소 실제 설치 검증은 아직 실행 전이며 완료로 보고하지 않는다.

Windows whole1145/0/30 fmt·clippy·test·release와 root 원본로그/소스/해시 검토 종료0. [전체검사](../validation/evidence/windows-msvc-installer-readable-compact-whole-20261012/root-review.json). 앱CPU9,695,744B·installer4,310,016B로 이전과 크기는 같고 새 dependency는 없다. Installer SHA `a81a20d42bdd6548014b661cddd546bae9c2308ed129e10e6cfaa1c84ad1d17c`. 실제 기본/최소 화면과 새5파일 설치 인수는 아직 남아 있다. 패키징 보호 테스트10개는 주 담당이 재실행해 통과했다.

기본 크기 실제 시험은 실패했다. preassert geometry가 요청800×620 대신 outer800×780/client784×741을 관측했다. WM_GETMINMAXINFO가 State 연결 전 null pointer에서 개발용 최소780을 적용하는 경로를 확인했다. [실제 실패](../validation/evidence/windows-msvc-installer-readable-compact-final-v2-20261012-install/root-measured-bootstrap-failure.json). 초기 null-state만 최소600으로 고치고 State 연결 뒤 packaged600/developer780 정책은 유지하는 수정이 다음 작업이다. 현재 소스 고정형 CloseWait 시험 종료 후 반영한다. 새 ZIP은 인수하지 않았고 README는 이전 설치 후보를 유지한다.

실제780px 초기 확대를 수정했다. State 연결 전 null-state bootstrap 최소600을 적용하고 연결 후 기존 packaged600/developer780은 유지한다. root 원본 SHA 대조·rustfmt 통과. 새 Windows whole·기본/최소 실제 설치 검증을 이어간다.

초기 최소 크기 수정 후 실제 기본800×620/client784×581·최소800×600/client784×561, 표시 child15개 전체 bounds 인수. 실제5파일 설치·재검증·journal SHA 보존·자연 app0/Job0와 현재27항목 native IDs 대조, root 기본/최소/설치/재검증 PNG를 확인했다. [최종 설치 검토](../validation/evidence/windows-msvc-bootstrap-closewait-final-20261012-install/root-independent-installed-review.json), [기본/최소 픽셀](../validation/evidence/windows-msvc-bootstrap-closewait-final-20261012-install/root-default-min-pixel-review.json). README는 실제검증한 새ZIP16,488,236B로 교체했다. 현대적 디자인 전체·physical hover/DPI·서명/cleanOS는 완료로 확대하지 않는다.
