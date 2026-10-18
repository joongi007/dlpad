# S11: v0.1.1 업데이트 지원 버전 공개

- 상태: in-progress
- 분류: release
- 우선순위: P1
- 요구사항: R08, R10, R11
- 담당/수정 범위: Root; README.md, site/index.html, docs/CICD.md, scripts/package_ci_preview.py, 관련 작업 기록; agent는 별도 로컬 검수 도우미만 담당
- 선행 작업: 9fe2045 업데이트 구현·Windows 로컬 전체 검증, 사용자 main/태그/설치파일 공개 승인

## 사용자 결과

GitHub v0.1.1 설치형·포터블 및 공개 서명 metadata를 제공한다. 기존 v0.1.0은 첫 지원 버전을 한 번 수동 설치한다.

## 계약과 범위

2026-10-18 사용자가 이전 자동 승인 거절 이후 이번 정확한 공개 범위를 승인했다. main과 새태그를 normal atomic push했다. 기존 태그/Release는 수정하지 않는다. 비공개키는 로컬 DPAPI 보관하며 외부 공급하지 않는다. 같은 release run의 원 build입력/notice/capture와 package artifact를 확인한 뒤 공개한다. 개인 Documents·실제영상 없이 fixture profile/가상desktop만사용한다.

## 인수 조건

새 hosted CI 전체 성공, 같은 SHA/run 원artifact·ZIP·manifest·PE·notice 검증, 실제 새설치/재검증·설치된CPU의privateGUI, operationalkey 로컬서명과 공개3파일원SHA 대조, 공개 API/다운로드/site갱신을 확인한다. 자동hosted서명과 Authenticode를 완료로 표현하지 않는다.

## 검증과 handoff

진행 중. source9fe2045c5b8a45ca0d2bf6cfcfd6c1f14a0512df; release run39132836927, main CI39132836883. 문서 job은 성공했다. Release Windows native의 기본 테스트/release 빌드 및 fixture strict lint/build는 성공했지만 첫 success 전환 case가 exit1로 실패해 package/draft는 실행되지 않았다. 원 Job은 natural empty/finalActive0/forcedCleanup=false이며 raw stderr와 receipt SHA를 대조했다. selection generation0이며 generic Other만 남아 최초 handshake와 준비 실패는 아직 구분할 수 없다. hosted 실패의 child-error/단계 관측을 보강한 후 원인을 확인한다. 로컬 7 PASS를 hosted 성공으로 대체하지 않는다. 공개전 v0.1.0은 유지한다. 로컬도우미/원로그는 ignored .local-tools/update-publication-20261018 아래 create-only로 보존한다.
