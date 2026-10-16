# S04: DownloadSession managed 실행 capability 전달

- 상태: implemented; integration-validation-pending
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; download.rs와 본 문서만
- 선행 작업: root ManagedExecution::run_tool/native helper, diagnostic_cache Deno inspect_with_managed

## 사용자 결과

Windows 다운로드가 관리 실행 capability를 받으면 yt-dlp/FFmpeg/Deno 버전 조회와 쿠키 없는 metadata/download를 managed guardian으로 실행한다.

## 계약과 범위

DownloadSession에 execution-only managed Option 추가; DB/options에 직렬화하지 않는다. 기존 constructor는 None. Windows managed run_tool은 VersionProbe/Metadata/Download typed stage 사용. 쿠키+managed는 기존 cookie runtime을 유지하고 paired journal runtime 미완료로 남긴다. 비Windows/None 동작 유지. root는 다른 파일 struct literals/shared caller, diagnostic_cache는 Deno 함수 담당. Cargo/다른 source 수정 제외.

## 인수 조건

version은 metadata capture=false로 stdout logs parser 유지, metadata=true는 기존 bounded JSON capture. cleanup typederror와 인증 omission 유지. 모든 managed reader coverage 완료나 mixed 인증 자동회수 가능을 주장하지 않는다.

## 검증과 handoff

download.rs source freeze SHA256 `68ed65fa56472d3dad8ec79894a4c69b0818af9e539674233027b55a903984f0`. DownloadSession managed optional execution-only field 및 이 파일의3 constructor None 구현. checked7args/checked_with_cookie6args로 전달한다. Windows version/FFmpeg는 VersionProbe capture=false, Deno inspect_with_managed, 쿠키 없는 metadata는 Metadata capture=true, download는 Download capture=false. 비Windows managed는 기존 실행 경로를 유지한다. cleanup typed 매핑/인증 omission을 변경하지 않았다. root가 다른 파일 struct literal 및 앱 capability 연결을 담당한다.

**mixed 인증+managed의 metadata/download는 기존 cookie helper만 사용**한다. 관리 journal에는 version/Deno/FFmpeg probe만 기록될 수 있으므로 그 receipt로 인증 reader lifetime 완료를 추론해서는 안 된다. native paired runtime 완료 전 startup 자동회수 활성화 금지 및 전체 managed reader coverage 미완료를 root에 전달했다. 이 패치가 paired native 보호나 안전한 startup 자동회수를 구현했다고 보고하지 않는다.

meaningful actual Windows fixture 후속은 공개 DownloadService+synthetic 도구에서 version/metadata/download 각 stage receipt 및 실제 성공·실패·취소/managed finish/원본sentinel 보존 확인이다. Deno/FFmpeg 지정 시 실제 VersionProbe receipt 추가 확인이 필요하다. pure argument 전달을 복제한 테스트는 추가하지 않았다. Cargo/rustfmt/native 실행 없음. 실제 compile/기존 download 회귀/native public caller 검증은 root 통합 이후 미검증이다.

주 담당 통합 검증: windows-managed-download-propagation-frozen-retry에서 실제 Windows GNU CPU 전체 fmt·Clippy(-D warnings)·workspace test·release 통과 및 현재 소스 SHA 일치를 확인했다. windows-managed-public-download-first는 공개 Repository에서 VersionProbe→Metadata→Download, Downloaded DB 저장, 실제 합성 artifact, 원 journal 정리·다음 실행 재사용을 통과했다. 주 담당이 DB의 job_json, 실제 파일 bytes, 3단계 marker, 원 도구 SHA/sentinel 및 9개 journal 파일 부재를 별도 읽어 대조했다. 초기 독립 SQL 조회는 state 컬럼 대신 실제 job_json 구조를 사용하도록 바로잡았다. Deno/FFmpeg 실제 지정 경로와 mixed 인증 paired runtime 및 startup 자동 회수는 아직 미검증/미완료다.
