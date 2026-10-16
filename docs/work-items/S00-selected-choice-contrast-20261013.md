# S00: 선택 버튼의 밝은 테마 글자 대비

- 상태: completed
- 분류: UX
- 우선순위: P1
- 요구사항: R06, R08
- 담당/수정 범위: root — crates/app/src/ui_components.rs 및 본 기록. advanced_units는 /tmp 후보, windows_auto는 독립 검토와 /tmp CI 준비만 담당한다.
- 선행 작업: 설치 실행 v11 전체 검사와 실제 성공·변조 거부 수명 시험의 소스 동결 완료.

## 사용자 결과

밝은 테마에서 선택된 탭과 버튼의 글자가 호버·누름 상태에서도 충분히 진하게 보인다.

## 계약과 범위

DESIGN의 작은 글자 4.5:1 기준을 따른다. 공통 choice의 활성화된 Light 선택 글자에 기존 ACCENT_PRESSED를 사용한다. 새 의존성·Cargo·schema 변경은 없다. 배경, 글꼴, 배치, 선택 상태와 이벤트 계약은 유지한다. 후보는 /tmp/dlpad-selected-choice-contrast-candidate-20261013에 보존한다.

## 인수 조건

실제 egui Context의 idle/hover/pressed/release 응답, 동일 ID·영역, 실제 AccessKit 선택 상태와 tessellated 글자·배경 대비를 두 테마에서 검증한다. 전체 fmt/clippy/test/release와 최신 패키지·격리 화면 검수를 통합 담당자가 직접 확인한다. 합성 포인터는 OS 물리 입력 인수를 대체하지 않는다.

## 검증과 handoff

아래 준비·실행 이력을 보존한다. 최종 scoped 인수는 v13 whole1,297/0/30, 설치 ZIP 검토 및 최신63개 화면의 팀 직접 검수·경로/SHA 독립 대조로 완료했다. Light 선택 글자가 읽히는 것을 확인했으며 실제 egui Context 두 테마4상태 응답/AccessKit/tessellated 글자 대비 회귀가 통과했다. OS 물리 pointer·reader 인수와 전체 UI/UX는 별도 미완료다. 최종 CPU 크기9,836,544B로 이 색상 수정의 크기 증가는0이다. [최종 검사](../validation/evidence/windows-msvc-installer-completion-copy-whole-20261013/root-review.json), [현재 직접 검수](../validation/evidence/current-completion-copy-app63-20261013/root-team-visual-review.json).

원 Light 대비는 idle 4.4903, hover 4.0351, pressed 3.5945이다. 제안된 기존 진한 색은 각각 6.4341, 5.7819, 5.1505이다. 이는 원 토큰의 sRGB 계산이며 실제 Rust 회귀·GUI 검증은 아직 실행하지 않았다. 현재 v11 소스는 수정하지 않았다.


실제 Windows baseline 회귀는 원 색상에서 exit101, 0성공/1실패로 실제 paint 대비4.490347525345249에서 실패했다. 원로그와 소스·자연 Job0를 독립 대조해 실패를 보존했다. 수정 후 전체 Windows fmt/clippy/test/release는1,297/0/30,43harness,source656으로 통과했고 root 원로그·3artifact·GNU·lock 검토도 통과했다. 새CPU cd8fe0fe…9,836,544B로 크기 증가는0이며 installer/CLI는v11과 byte-identical이다. 두 테마에서 실제 selected idle/hover/pressed/release paint·응답·AccessKit 회귀가 성공했다. native 최신 픽셀 검수와 최종 설치 패키지는 다음 단계이다. [baseline 원실패](../validation/evidence/windows-msvc-selected-choice-contrast-baseline-red-20261013/root-failure-review.json), [전체검사](../validation/evidence/windows-msvc-selected-choice-contrast-whole-20261013/root-review.json).
