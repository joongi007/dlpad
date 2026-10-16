# S04: Deno YouTube 지원 구성요소 공급 후보 실증

- 상태: done (Linux runtime 후보; 관리 채택/YouTube 통합은 미완료)
- 분류: test
- 우선순위: P0
- 요구사항: R04, R05, R11
- 담당/수정 범위: library_identity — 신규 scripts/deno_supply_probe.py, 전용 validation 증거
- 선행 작업: 기존 공급 증거의 공식 Deno v2.9.7 Linux ZIP pin

## 사용자 결과

선택 JS runtime의 실제 버전·동작·용량·실행/캐시 비용을 공개하고 timeout/cancel을 확인한다. yt-dlp bundled EJS와 별개이며 YouTube 실제계정·사이트 지원 완료를 주장하지 않는다.

## 계약과 범위

공식 release API/checksum/source/license와 EJS guide를 재확인한다. 프로젝트 소유 새 임시 folder/DENO_DIR만 사용하고 bounded ZIP regular-file allowlist로 deno·notice만 추출한다. Archive SHA와 executable SHA를 별도로 기록한다. 원격 코드/네트워크/쿠키/실제계정/미디어 없이 오프라인 합성 JS와 argv-only 소유 process group을 검증한다. 제품/sharedschema/Cargo/기존 공급문서는 수정하지 않는다.

## 인수 조건

- 공식 size/checksum/API/실제 ZIP SHA가 일치하고 실행 파일별 SHA를 기록한다.
- 경로 탈출·symlink·duplicate·크기 상한의 ZIP 실패 경로를 시험한다.
- version/offline fixture/무한루프 timeout/명시 cancel을 실제 실행한다.
- 전용 cache budget, 설치·CPU/RSS 비용을 기록하고 OS/source/notice/trust 미완비를 후보 gate로 남긴다.

## 검증과 handoff

공식API/checksum/실제ZIP41596794bytes와SHA 일치, 추출deno95830104bytes/component SHA 별도기록. 실제2.9.7 오프라인 합성JS pass, 무한루프Timeout1.016초/명시Cancel0.260초 회수, 전용cache296088bytes와CPU/RSS 측정. 추출/cache self-test8개 통과, 문서157개 통과. [실제 결과](../validation/deno-supply-validation.md), [원시 증거](../validation/deno-supply-probe.json)에 보존했다. bundled EJS/실제사이트/3OS/source·thirdparty notices/trust/관리activation은별도gate로남긴다. 외부 사용자 Deno cache를 읽거나 삭제하지 않았다. 기존 .git 보존.
