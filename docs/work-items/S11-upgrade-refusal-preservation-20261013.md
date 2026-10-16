# S11: 미지원 업그레이드 요청과 기존 설치 자료 보존 검수

- 상태: completed
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: advanced_units — /tmp 합성 collector/후속후보; root — 실행/통합/문서, 기존 제품 source 미변경
- 선행 작업: 최신 Windows whole와 동일5파일 payload

## 사용자 결과

아직 지원하지 않는 upgrade/rollback 명령과 기존 경로 재설치 요청이 잘못된 성공을 반환하거나 기존 파일을 바꾸지 않는지 확인한다. 실제 upgrade/rollback 구현 완료와 구분한다.

## 계약과 범위

새 합성 root의 journal bundle과 설정/partial/가짜 credential sentinel을 사용한다. current whole·CLI·payload/source pin, 정확한 typed refusal/exit, 순차 Windows native ID/namespace/hash 보존을 검증한다. 실제 사용자 파일·계정·앱 실행·제거·외부 전송은 사용하지 않는다. readonly 두 bundle 비교 CLI 후보는 /tmp에만 두고 이번 UI source batch에는 채택하지 않는다.

## 인수 조건

미지원 두 명령 InvalidInput, 동일 root AlreadyExists, unknown 삽입후 inspection InvalidData를 관측하고 모든 원 ID/SHA/namespace와 journal consistency를 보존한다. 순차 검증을 atomic snapshot·역사 소유권·upgrade/rollback/crash commit 인수로 승격하지 않는다.

## 검증과 handoff

/tmp/dlpad-upgrade-rollback-audit-20261013에 collector 및 후속후보를 준비했다. 순수 guards4 PASS/AST/후보 rustfmt는 준비 증거다. 실제 Windows 실행은 최신 payload 생성 후 root가 수행하며 현재 미실행이다.

실제 root 실행: current whole-v7/payload-v5 고정 후 Windows13명령을 실행했고 미지원 upgrade/rollback InvalidInput, 같은root AlreadyExists, unknownfile 검사 InvalidData 및 원 ID/SHA/namespace/3sentinel/journal consistency 보존을 확인했다. root가13원로그의 기대exit0/1·privateJobterminal·source/원pin·native전후를 독립 대조해 PASS_SCOPED_SYNTHETIC_REFUSAL_PRESERVATION으로 기록했다. 실제 upgrade/rollback/crashcommit 기능은 여전히 미구현이며 후속 readonly 비교 patch도 채택하지 않았다. [독립 검토](../validation/evidence/windows-upgrade-refusal-v5-20261013/root-review.json).
