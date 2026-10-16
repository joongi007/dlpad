# S11: 컴파일 포함 리소스 source binding

- 상태: completed (scoped)
- 분류: validation
- 우선순위: P1
- 요구사항: R08, R11
- 담당/수정 범위: root — scripts/windows_current_workspace_probe.py, scripts/test_windows_snapshot_binding.py, .github/workflows/ci.yml 및 공유 source계약/후속검증; installer_batch — /tmp 격리실험 준비
- 선행 작업: 현재v17 whole/package/native sourceguard 종료

## 사용자 결과

배포와 검증 기록이 Rust 코드에서 포함하는 리소스도 정확히 고정하고 격리된 실험에서 동일한 입력을 사용한다.

## 계약과 범위

현재snapshot656은Rust/Cargo/appassets/vendor/host의정의된집합이다. root가독립격리실험 export준비를검토하며 engine/src/tools.rs252 include_str!(tools/staging_schema.sql)이그집합밖임을확인했다. SQL/DBschema를바꾸지않는다. 현재네명령실제통과나656정의된pin검증을부정하지않으며allbuildinput완결을주장하지않는다. active sourceguard중snapshot함수/공유검증계약을바꾸지않는다. 별도격리실험에서는명시SQL추가pin/복사/전후검증을기록한다.

## 인수 조건

실제literal include_str/include_bytes의경로목록을대조하고현재누락리소스를새source domain에포함한다. sourcecount변경은전체새proof/collector의정확계약과조정하고원656증거를새domain으로소급하지않는다. SQL변조/누락과공유source불일치실패를거부하는실질검증을한다.

## 검증과 handoff

정적준비시누락을발견했으며아직격리Cargo/export를실행하지않았다. sourcebinding후속은현재17native/sourcefreeze이후다. 원SQL/data/lock/schema는그대로보존한다.


root가actual snapshot함수의순수/tmpfixture 회귀를독립재실행했다: original3시험2FAIL(exit1), requiredSQL후보3/0(exit0). 실제SQL/제품collector는바꾸지않았다. Frozen후보HANDOFF의현재657표현은오류이며현재17정의domain은656이다. future추가는actual측정으로확인한다. [독립준비검토](../validation/evidence/compiled-resource-source-binding-preparation-20261013/root-review.json).


2026-10-13 root 통합: 원 source freeze 종료 후 제품 코드에 채택했다. 완료 인수는 새 전체 Windows 검증·설치본 검수 이후다. scripts/windows_current_workspace_probe.py의 required path에 기존 SQL만 추가했다. scripts/test_windows_snapshot_binding.py의 실제 snapshot 변조·누락·기존 자산 회귀 3/0 통과. 새 source domain은 실제 측정657이며 원656 증거를 소급 변경하지 않는다. SQL 내용·schema·Cargo.lock은 보존했다.


새 SQL 누락·변조 회귀를 .github/workflows/ci.yml의 독립 Python 검사 단계에 연결했다. 로컬 실제 3/0 통과이며 hosted 실행은 유효 Git/remote 이후 미검증이다. 네 Rust 전체 명령 actual1327/0/33/source657은 별도 root 독립 검토를 통과했다.


v18 통합 인수: 네 Windows 실제 명령 1327/0/33·43harness·source657, 실제 현재3PE·lock·원로그 독립 대조, 새 ZIP·정상 설치/앱 자연 종료·변조 거부·canonical27ID·앱63 및 설치8 PNG 직접 검수를 완료했다. 공유 Cargo/schema/의존성 변경은 없다. [현재 whole](../validation/evidence/windows-msvc-keyboard-diagnostic-help-whole-20261013/root-review.json). 추가 Settings distinct-commit semantic FAIL은 별도 후속 작업이며 이 scoped 성공으로 덮지 않는다. 전체 P0·물리 IME/reader/OS DPI·full UI 인수는 미완료다.
