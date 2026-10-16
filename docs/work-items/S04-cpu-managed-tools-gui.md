# S04: CPU 관리 도구 격리 GUI 검증 준비

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R06, R08, R11
- 담당/수정 범위: library_identity — 신규 scripts/cpu_tools_probe.py 및 전용 문서
- 선행 작업: root 관리 패널 통합/소스 동결/새 CPU binary READY

## 사용자 결과

합성 관리 버전 두 개로 목록 갱신·보관 활성화·보관 제거·active/외부bytes 보존·Use 적용을 실제 네이티브 UI에서 관측한다.

## 계약과 범위

owned Xvfb/private DBus 및 temp fixture만 사용하며 visible desktop fallback은 없다. seed는 managed_retained_probe의 manifest/owner/activation 계약을 재사용한다. 공식 공급 pin은 managed_tools_probe 기록과 구분하여 합성fixture의 --version/hash를 실제배포 검증으로 주장하지 않는다. READY 이전 GUI 실행 금지. 신규 네트워크 다운로드·계정·posting 없음.

## 인수 조건

AT-SPI enabled action으로 refresh/select/remove/use를 관측하고 read-only SQLite 및 exactfixturebytes로 결과 확인. 빌드SHA before/after 고정. CloseWait는 owned activation.sqlite EXCLUSIVE lock을 통해 bounded 진행 작업을 만든 뒤 닫기/중단후종료를 관측한다. 실제 설치 cancel은 synthetic inventory만으로 검증할 수 없어 별도 opt-in 실제 다운로드 인수로 유지한다.

## 검증과 handoff

현재 준비 단계다. py_compile/인자 검증은 GUI 성공이 아니며 READY 후 실행 및 실패 raw 보존이 필요하다.

합성 bounded Linux GUI 범위 통과. 초기 collector 실패와 fresh Textfix 통과를 모두 보존했다. 실제 install cancel/3OS/전체S04는 완료 범위가 아니다. 전용 validation에 exactSHA·CloseWait·미검증을 기록했다.
