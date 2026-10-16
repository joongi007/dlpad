# S04: 제품 managed 실행 세션 전달

- 상태: in-progress
- 분류: integration
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: root: app/accessible.rs 및 engine/tests의 DownloadSession fixture, engine/examples/windows_cookie_startup_seed.rs; download.rs와 deno_runtime.rs는 별도 하위 작업 담당
- 선행 작업: 실제 managed capture helper 인수; DownloadSession.managed 공유 계약

## 사용자 결과

관리 도구를 선택한 제품 다운로드의 실행 세션을 전달하고 모든 버전 확인 및 비인증 metadata/download 실행을 managed guardian으로 연결한다.

## 계약과 범위

실행 capability는 options/DB/argv snapshot에 직렬화하지 않는다. 기존 cookie fixture는 managed None을 명시한다. managed/cookie 동시 metadata/download는 원 Job에 두 journal을 바인딩하는 후속 paired runtime을 필요로 한다. 그 전에는 기존 인증 경로를 유지하지만 전체 reader coverage 완료나 startup 자동 회수 권한을 주장하지 않는다. shared API 및 모든 생성자 조정은 root가 통합한다.

## 인수 조건

기존 비관리 및 인증 실패 회귀가 보존되고 관리 버전/비인증 download의 실제 Job·receipt 종료 경로가 실행된다. 코드 컴파일만으로 native 제품 실행을 완료 처리하지 않는다.

## 검증과 handoff

구현과 통합 검증은 아래 기록한다. 자동 회수는 모든 reader의 paired 연결 후 활성화한다.

하위 download/Deno 구현 인수 후 모든 생산 호출을 inspect_with_managed로 전환했고 이전 inspect(None) wrapper는 기존 단위 시험용 cfg(test)로 유지했다. 실행 capability는 Windows에만 새 reader 경로를 사용하며 인증 결합 미완료를 코드와 작업 문서에 명시한다. 실제 제품 경로의 fixture 인수는 후속이다.

첫 native 전체 Clippy는 이전 inspect wrapper가 Windows lib test에서는 사용되지 않는 dead_code로 실패했다. 실제 wrapper를 사용하는 기존 시험의 Linux x86_64 cfg와 일치하도록 제한하여 수정했다. 실패 로그 windows-managed-download-propagation-first는 보존한다.

주 담당 통합 검증: windows-managed-download-propagation-frozen-retry에서 실제 Windows GNU CPU 전체 fmt·Clippy(-D warnings)·workspace test·release 통과 및 현재 소스 SHA 일치를 확인했다. windows-managed-public-download-first는 공개 Repository에서 VersionProbe→Metadata→Download, Downloaded DB 저장, 실제 합성 artifact, 원 journal 정리·다음 실행 재사용을 통과했다. 주 담당이 DB의 job_json, 실제 파일 bytes, 3단계 marker, 원 도구 SHA/sentinel 및 9개 journal 파일 부재를 별도 읽어 대조했다. 초기 독립 SQL 조회는 state 컬럼 대신 실제 job_json 구조를 사용하도록 바로잡았다. Deno/FFmpeg 실제 지정 경로와 mixed 인증 paired runtime 및 startup 자동 회수는 아직 미검증/미완료다.
