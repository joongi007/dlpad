# S11: 현재 설치·검증 안내와 개발 이력 정리

- 상태: complete
- 분류: docs
- 우선순위: P0
- 요구사항: R10, R11
- 담당/수정 파일: root README.md·DEVELOPMENT_HISTORY.md·실제 현재 artifact/증거 링크 통합; release_preparation은 /tmp 문서 후보만 작성한다.
- 의존성: 실제 설치 ZIP 및 검증 기록. 새 수동 도구 설정 설치본은 GUI 인수가 끝난 뒤에만 현재 사용자 배포 안내로 반영한다.

## 범위

Git 준비 시 현재 설치 방법·시험 범위·문서 읽는 순서를 빠르게 찾도록 README를 정리한다. 오래된 날짜별 결과는 원문과 링크를 같은 root의 개발 이력 문서로 보존한다. 제품 요구사항·실패 자료·공급 원 공지를 삭제하거나 재해석하지 않는다.

## 인수 조건

현재 실제 제공 가능한 ZIP과 실험 중 후보를 구분한다. 이전 binary의 비용/GUI/설치 결과를 새 후보에 적용하지 않는다. 실제 실행되지 않은 CI/서명/updater/8시간 시험은 완료로 쓰지 않는다. 모든 local link와 기존 원문 보존을 check_docs 및 root 차이 검토로 확인한다.

## 검증과 handoff

README·개발 이력·export allowlist 통합을 완료했다. 아래 진행 기록과 최종 검증을 구분한다.

## 주 담당 통합 검증

README 설치 안내를 정리하고 전체 원 README를 DEVELOPMENT_HISTORY.md에 verbatim 보존했다. AGENTS 문서 순서를 일치시키고 실제 도구 경로 입력 동작으로 안내를 교정했다. scripts/export_source.py allowlist에 이력 문서를 포함했다. `python3 -m unittest discover -s scripts -p test_export_source.py`: 5 passed. 새 설치 ZIP·GUI 최종 검증은 진행 중이며 기존 후보 안내를 유지한다.

프로젝트 문서 검사: 894 documents/114 hash-matched notices PASS. 실제 create-only source export `/tmp/dlpad-source-export-readme-20261011`에 DEVELOPMENT_HISTORY.md 및 README history 링크 포함, private evidence omission rewrite 적용을 root 확인했다. export의 문서 검사도 838 documents PASS다(제외된 private notice 0).

최신 실제 인수된 설치 ZIP manual-tool-preferences를 README와 사용자 실행 안내에 반영했다. whole1055/GUI5phase/설치5file scope와남은remoteCI/updater/soak/OS인수를 명시했다. 최종문서검사후docs수직단위를완료한다.

최종 local `python3 scripts/check_docs.py`: 894 documents/114 notices PASS. 실제sourceexport838문서와history omission 정책, export5tests 및 Git1,806files commit검사를 통과하여 문서정리 수직단위를완료했다. 전체P0/릴리스완료상태와구별한다.
