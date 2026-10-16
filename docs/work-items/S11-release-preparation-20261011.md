# S11: Git·Windows CI·업데이트 사전 검증 준비

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R05, R08, R10, R11
- 담당/수정 범위: root 통합; .github/workflows/ci.yml, scripts/export_source.py, scripts/validate_update_candidate.py, 두 test 스크립트
- 선행 작업: 기능 인수, 정상 Git/remote 및 CI 실제 성공은 앱 updater 실행의 선행 조건

## 사용자 결과

Windows 검사를 기본 CI 대상으로 하고 다른 OS는 수동 선택으로 남긴다. 새 폴더로 소스만 내보내는 개발 스크립트와 unsigned 업데이트 후보의 읽기 전용 정책 검사를 준비한다. 앱 업데이트 실행과 공개 배포는 아직 제공하지 않는다.

## 계약과 범위

기존 빈 .git을 수정하지 않는다. source export는 기존 출력·symlink·프로젝트 내부 출력을 거절하고 바이너리·evidence·개인 도구 폴더를 제외한다. 복사본의 제외 자료 inline 링크는 미포함 안내 한 문서로만 연결하고 원 링크·원 SHA·변환 SHA를 schema2 receipt에 기록한다. 원본 문서는 변경하지 않는다. 이 export가 과거 기능 검증 근거나 게시 안전성·CI 실제 성공을 만들지 않으며 게시 전 내용 검토가 필요하다. Python은 개발 보조 도구이며 Rust 제품 런타임 의존성을 추가하지 않는다. Cargo.lock 변경 없음.

업데이트 JSON은64KiB, 패키지256MiB 상한이며 duplicate/unknown 필드, 버전·채널·MSVC target·HTTPS 호스트·SHA 형식을 검사한다. 서명 검증이 없으므로 release_ready=true를 항상 거절한다. 다운로드·교체·rollback 및 원본 설치 journal 권한은 구현 범위가 아니다.

## 인수 조건

출력 충돌·symlink 거절, 원 .git 보존, manifest 손상·버전 역행·다른 채널/호스트·unsigned readiness 승격 거절 회귀를 실제 실행한다. CI는 remote 실행 결과 없이 성공으로 기록하지 않는다.

## 검증과 handoff

2026-10-11 후보를 root가 검토·통합했다. 실제 Python 테스트와 문서 검사를 아래에 기록한다. Git remote 주소는 사용자 응답 대기이며 독립 구현을 계속한다. 서명 trust root·채널 endpoint·업데이트 crash 복구·원본 설치 identity·사용자 데이터 migration·정식 notice·clean OS 인수는 남는다.

Root 실제 검증: `python3 scripts/test_export_source.py` 2 tests PASS; `python3 scripts/test_update_candidate.py` 3 tests PASS. `python3 scripts/check_docs.py` 871 documents/114 preserved notices PASS(이후 작업 문서 추가로 개수 변동 가능). 원 Git/remote 외부 게시와 CI remote 실행은 수행하지 않았다.

2026-10-11 root source export 증거 미포함 안내 변환을 통합했다. raw validation 자료·로그·ZIP은 제외하고 제품 빌드용 resource 소스는 유지한다. 하위 후보의 실제 /tmp export docs818 PASS는 구조 검증이며 root 추가 실행은 아래 후속 기록한다.

Root 실제 추가검증: `python3 scripts/test_export_source.py` 5tests PASS; `python3 scripts/export_source.py /mnt/d/project/dlpad /tmp/dlpad-root-source-export-20261011` exit0; 복사본 `scripts/check_docs.py` 818documents PASS. receipt의 모든 원 SHA/최종 SHA/크기를 root가 독립 재계산해 일치했다. 외부 Git 게시나 CI 실행은 하지 않았다.

후속 검증: `python3 scripts/test_update_candidate.py` 3tests PASS를 다시 확인했다. 현재 원 Git status는 exit128이며 기존 .git을 보존했다. 로컬 source bundle과 unsigned 설치본이 있어도 원격 Actions 실행 및 runtime updater 인수가 성립하지 않는다.
