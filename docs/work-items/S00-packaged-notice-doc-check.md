# S00: 패키지 원본 notice와 작성 문서 검사 구분

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root — scripts/check_docs.py, 이 작업 문서
- 선행 작업: 기존 사용자 ZIP 추출 및 새 UI 후보 문서 검사

## 사용자 결과

패키지의 원본 라이선스 문구를 수정하지 않고 프로젝트 작성 문서를 검사한다.

## 계약과 범위

기존 extracted 개발 ZIP의 moxcms/pxfm LICENSE.md에는 원본 마지막 개행이 없어 docs checker가 실패했다. docs/validation/evidence 아래 candidate-manifest와 license-inventory가 있는 bundle의 licenses 경로에서 inventory SHA와 실제 bytes가 정확히 같은 원문만 작성 문서 형식/링크 검사에서 제외한다. 불일치·미선언 파일·잘못된 metadata는 제외하지 않는다. 원 라이선스나 ZIP을 변경하지 않는다. Rust/Cargo/제품 기능 변경은 없다.

## 인수 조건

실제 두 원문 SHA가 일치하는 것을 확인한다. 합성 임시 bundle에서 정상 notice는 제외하고 변경 bytes·미선언 문서·metadata 실패는 제외하지 않는 것을 확인한다. 전체 docs checker를 통과하고 제외 수를 보고한다.

## 검증과 handoff

최초 `python3 scripts/check_docs.py`는 원 LICENSE 두 개의 missing final newline으로 exit1이었다. 원파일 보존 후 정확 SHA 조건으로 수정·재검증한다.

실제 원문 두 개와 합성 정상/bytes 변경/미선언/malformed metadata/경로 이탈 거절을 inline Python으로 검증하여 PASS였다. 원 LICENSE bytes와 이전 ZIP은 변경하지 않았다. `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_docs.py`는 exit0:852 작성 문서 링크/형식 PASS, 정확 SHA 일치 패키지 원자료19개 보존·제외를 보고했다. 외부 URL/heading anchor 및 라이선스 법적 완비를 검사했다는 의미는 아니다.
