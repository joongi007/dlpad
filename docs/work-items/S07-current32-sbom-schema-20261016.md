# S07: 현재 선택 SBOM의 공식 schema 검사

- 상태: done-scoped
- 분류: validation
- 우선순위: P0
- 요구사항: R11 / S07
- 담당/수정 범위: Root; 새 B/current32-sbom-schema-root-v1, 새 전용 evidence와 이 문서
- 선행 작업: current32 선택 SBOM artifact와 원 입력 동결

## 사용자 결과

배포 구성요소 목록이 도구로 읽을 수 있는 CycloneDX JSON 문법인지 실제 공식 schema와 대조한다.

## 계약과 범위

공식 CycloneDX 저장소의 version1.6 JSON schema를 2MiB·20초 HTTPS 읽기 상한으로 새 namespace에 보존한다. 임의 참조의 추가 network 접근은 거절하고 필요한 schema만 명시 고정한다. 기존 jsonschema를 사용하며 의존성/제품/lock 변경 없다. schema 적합성이 라이선스·linked code·공급 trust 완결을 증명하지 않는다.

## 인수 조건

schema 원 URL/SHA/bytes/validator 버전·원 SBOM SHA를 기록한다. 공식 schema로 실제 artifact 검증; 잘못된 type/version 음수 fixture 거절. 실패 원본 보존·기존 artifact 덮기 없음.

## 검증과 handoff

공식 CycloneDX/specification의1.6/schema 공개 HTTPS에서3파일을 다운로드했다. 첫 sandbox DNS 실패는 다운로드0였고, 사용자 허용 의존 자료 다운로드 범위의 require_escalated 실행은 실제0이었다. 자동 승인 거절/우회는 없었다. 원3schema SHA/bytes/URL을 fetch-result.json에 기록했다.

`python3 -B .local-tools/session-recovery-20261013/current32-sbom-schema-root-v1/validate-v2.py` 실제 exit0: jsonschema3.2.0/Draft7+FormatChecker,258components 실제 artifact 공식 schema PASS·invalid component type/정수 specVersion 음수2개 거절. 모든 외부 $ref는 pin된3schema만 허용하고 추가 네트워크 해석은 금지했다. 초기 validate.py의9.9 문자열 음수는 공식 schema가 specVersion을 enum으로 제한하지 않아 실패했다. 이를 공식 schema가 보장하지 않는 범위로 기록하고 version 타입 음수로 수정했다. 정확한1.6 문자열은 별도 inventory 원입력 validator로 고정한다. [Root 검사](../../.local-tools/session-recovery-20261013/current32-sbom-schema-root-v1/root-review.json).

Root는 별도 원 sbom.py --check도 실제0으로 재실행해258/456/appendix43·byte재현·음수2거절을 확인했다. 이 결과는 서명/공급 provenance/embedded source/법적완결/최종 P0를 증명하지 않으며 기존 v32 ZIP에 SBOM이 들어 있다고 주장하지 않는다.


Root는 선택 SBOM/공식schema3개/validator와 실제검토를 [고정 evidence](../validation/evidence/current32-selected-sbom-root-20261016/root-integration.json)에 create-only 복사하고 각SHA/bytes를 다시 대조했다. [검토용 SBOM](../validation/evidence/current32-selected-sbom-root-20261016/selected-current32.cdx.json)은 기존ZIP과 별도 sidecar다.
