# S04: Windows public managed download native fixture

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 신규 examples/windows_managed_download_fixture.rs, 이 문서만
- 선행 작업: root DownloadSession.managed + Windows managed guardian/receipt +finish/pristine cleanup

## 사용자 결과

public DownloadRepository에서관리합성yt-dlp의실제VersionProbe/Metadata/Download3stage를실행해Downloaded journal/실제artifactbytes를확인하고관리finish후reuse까지검증한다.

## 계약과 범위

selfexe를새원WindowsTEMP directchild/data/managed-tools/yt-dlp/version/target/yt-dlp.exe에복사해originalSHA/native관리manifest로선택. guardian_entry/register_self_guardian 필수. 자식argvversion은ignoreconfig/noplugins/--version정확대조,metadata는dump-single-json,download는ownedpartial --paths와고정job.%(ext)s 템플릿/DLPAD_ARTIFACT JSON프로토콜. 실제network/media/cookie/Deno/FFmpeg없음. source/sentinel보존,hash64KiBbuffer/256MiB/evidence16KiBcap. failure不確定helper때temp삭제않고ownedroot를stderr표식과성공JSON에넘겨collector소유로보존한다. root는그증거root만이후정리한다.

## 인수 조건

실제3자식stage marker+publicmanagedfinish성공(원receipt검증의간접API증거,별도lockedjournalhandle열지않음),JobDownloadedpersisted/artifactexactbytes,9journalfiles+pendinggone,새scopefinish성공/sourcehash/sentinel보존. selectedDeno/FFmpeg/mixedcookie와actualcrash/자동startup회수는후속이며미검증.

## 검증과 handoff

/tmp초안 작성/ownfmt. root게시허용후신규example게시 READY; Cargo변경없이root실제Windowsbuild/run( --fixture-run 새result.json ). 현재compile/native미실행이며완료claim없음.

주 담당 통합 검증: windows-managed-download-propagation-frozen-retry에서 실제 Windows GNU CPU 전체 fmt·Clippy(-D warnings)·workspace test·release 통과 및 현재 소스 SHA 일치를 확인했다. windows-managed-public-download-first는 공개 Repository에서 VersionProbe→Metadata→Download, Downloaded DB 저장, 실제 합성 artifact, 원 journal 정리·다음 실행 재사용을 통과했다. 주 담당이 DB의 job_json, 실제 파일 bytes, 3단계 marker, 원 도구 SHA/sentinel 및 9개 journal 파일 부재를 별도 읽어 대조했다. 초기 독립 SQL 조회는 state 컬럼 대신 실제 job_json 구조를 사용하도록 바로잡았다. Deno/FFmpeg 실제 지정 경로와 mixed 인증 paired runtime 및 startup 자동 회수는 아직 미검증/미완료다.
