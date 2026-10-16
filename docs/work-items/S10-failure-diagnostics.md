# S10: 실패 설명과 진단 자료 전달

- 상태: proposed
- 분류: feature
- 우선순위: P0 (로컬 보고서), P1 (알려진 문제), P2 (AI API)
- 요구사항: R13, R08
- 담당/수정 범위: 미배정; engine 진단/정제, app 실패 상세, 진단 schema와 fixture
- 선행 작업: S02 Attempt와 프로세스 출력 수집; P1 업데이트 제안은 S04 필요

## 사용자 결과

실패한 다운로드에서 단계·근거를 확인하고 개인 정보를 줄인 보고서를 AI나 개발자에게 전달한다. 진단 과정에서 원래 다운로드·도구·파일이 변경되지 않는다.

## 계약과 범위

[ADR-0004](../adr/0004-diagnostics.md)와 [진단 설계](../DIAGNOSTICS.md)를 따른다. 번호는 S09 이후 개발 순서를 뜻하지 않는다. P0 부분은 S02 이후 개발하여 S07 전에 완료한다. 알려진 문제 조회와 API 연결은 별도 후속 구현이다.

## 인수 조건

실패 상세→정제 보고서→미리보기→복사/저장을 연결한다. sourceURL 예시, 다른 원인의 동일 예외, 민감 URL·쿠키, 깨진/큰 MPD, offline fixture를 포함한다. 온라인 전송 없이 P0 완료 가능해야 한다. 카탈로그 상태와 PR/배포 상태를 혼동하지 않는다.

## 검증과 handoff

core redactor/MPD parser, engine create-only export 및 개발 UI의 미리보기·복사·저장·로그 수정/제외·로컬 MPD 첨부를 구현했다. [구현 작업](S10-implementation.md)과 [남은 게이트](../validation/P0-status.md)를 따른다. 합성 secret fixture로 검증했으며 실제 OS clipboard와 전체 실패 흐름의 검수는 남아 있다.

2026-10-11 최신 GUI 회귀 준비: glyph 좌표는checkbox내부지만 private postedWMclick이 ToggleState.On을 만들지못해 진단 입력 전 FAIL을 보존했다. 제품checkbox결함이나 물리pointer검증으로 해석하지 않는다. 정확한 privateprovider TogglePattern을1회 호출하고freshOn readback을2초bounded확인하는 수집 후보로 변경, 현재비용측정 후 실행 예정이다. 모든 진단3입력 실제 검증은 아직 완료로 표시하지 않는다.

TogglePattern 전제조건On 실제통과, 합성실패보고서생성·cookieclear뒤 진단저장경로의pointerfocus에서FAIL을보존했다. field수집기에settle/freshgeometry전후근거와privateprovider SetFocus 경로를별도표시하며편집readback검증을유지했다. pointer/물리입력성공으로보고하지않는다. 재검증예정.

## 최신 인증 설정 경로 재시험

windows-msvc-s10-current-large-expanded-json-20261012는 실제 접힌 인증 설정 공개 및 URL·쿠키 입력 readback까지 성공했으나 시작 버튼 Invoke unavailable/disabled로 종료2였다. 내부 앱 forced cleanup이 있으므로 진단 인수 FAIL이다. 원본 로그와 fixture를 보존하며 최신 버튼 재조회 및 작업 시작 gate를 다음 조사로 남긴다.

최신 windows-msvc-s10-current-large-footer-invoke-json-20261012는 URL 입력→실제 bypass 선택→인증 설정 펼침→쿠키 readback→현재 enabled footer button Invoke로 정상 진행했다. 원앱 exit0/Job active0/workerdrain/private screen/input desktop 보존, JSON1,841,855B SHAdefce1fde792980e24cf1810ffc716bbe25dc7acc77c89d58949159ef10f36ba를 독립 대조했다. 정제·원sourcecookie/sentinel SHA·SQLite Failed1/Attempt1/Artifact0 및 DB bytes보존을 확인했다. terminal PNG 직접 검토했으나 전체225페이지 시각·정확2MiB 도메인 fixture·peakmemory·실행중CloseWait·Windows clipboard는 미검증이다. 이전 실패 원본은 보존했다.
