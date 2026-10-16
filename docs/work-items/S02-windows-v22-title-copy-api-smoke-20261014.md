# S02: 제목 파일명 다운로드와 이름 지정 복사의 Windows 실행 검증

- 상태: done (공개 API 합성 실행 범위)
- 분류: integration-test
- 우선순위: P0 다운로드 검증 / P1 복사 검증
- 요구사항: R03, R04, R08, R11
- 담당/수정 범위: installer_batch — 독립 Rust/C# fixture와 실행 도구; root — 인터페이스 검토·실행·증거 통합·문서. 제품 소스는 동결한다.
- 선행 작업: v22 source669 전체 검사·원 실행 파일 세 개·Root 검토, 같은 컴파일 입력의 소유 복사본

## 사용자 결과

영상 제목을 유지한 파일명으로 다운로드하고, 파일 목록에서 새 이름으로 복사할 때 원본과 기존 대상 파일을 보존하는지 실제 Windows API로 확인한다.

## 계약과 범위

기존 engine manifest·잠금파일·의존성을 그대로 사용한다. 새 예제만 소유 소스 복사본에 추가한다. 합성 도구는 소유 임시 폴더에서 CSC로 컴파일하며 실영상·네트워크·계정은 사용하지 않는다. 제품의 DownloadRepository와 Store, file_export 공개 API를 호출한다. shell 문자열 실행·전역 도구·제품 Cargo/schema 변경은 없다. 실제 GUI 복사·폴더 대화상자·cross-volume·진행 중 취소·자막·강제 V3 복구는 별도 인수다.

## 인수 조건

한글·emoji·Windows 예약 제목의 최종 파일명, 내용 SHA와 native identity를 확인한다. 저장된 파일명 옵션과 실행 의도가 다르면 작업 시도 전에 거절한다. 기존 다운로드·복사 대상과 원본을 덮어쓰지 않는다. 취소된 요청·예약 이름·오래된 원본 identity를 거절한다. 같은 폴더 복사는 다른 target identity를 가지며 같은 내용을 유지해야 한다. 원 도구와 sentinel, 현재 source669 및 실행 파일 세 개를 전후 대조하고 private Job/desktop의 자연 종료를 확인한다.

## 검증과 handoff

실제 명령은 `native-readable-copy-smoke-v1/run_root.py`의 prepare→build→run에 현재 whole proof/Root review/source669를 전달했다. 소유 export2,013파일과 Windows 예제 빌드는 통과했다(native compile12.68초, 자연0/Job0). 첫 실행은 합성 CSC compile0 뒤 관측 fixture에서 Win32 AccessDenied5/자연1로 실패했다.

제품의 open_regular_no_reparse는 FILE_READ_ATTRIBUTES만 연다. 시험이 이 metadata handle에서 내용 Read를 시도한 계약 위반을 확인했다. 원 v1 소스·exe·fixture·FAIL을 보존하고 새 v2 예제에서 별도 nofollow read handle의 동일 native identity를 검사하도록 수정한다. 제품 파일 신원 API나 권한은 바꾸지 않는다. 제목 다운로드·복사의 실제 인수는 아직 완료하지 않았다. [첫 실행](../validation/evidence/current22-readable-copy-api-run-20261014/result.json).

v2 prepare와Windows예제빌드는통과했다. 두번째실행은원도구의별도readHandle 신원/내용관측과CSCcompile0을통과한뒤metadata단계에서합성.NET도구가0xe0434352로종료했다. 기존v1핸들계약오류와구분한다. bounded오류로그는Exception.ToString 실패문구만포함하므로 CLR내부원인은아직미확정이다. 같은v2 Rustbinary를유지하고새합성CSharp입력만계측하는v3로좁힌다. 원v2FAIL을보존하며제품다운로드성공으로승격하지않는다. 독립된순수revision정책/metrics API시험은합성도구의성공에기능적으로의존하지않으므로원소스·빌드·provider·실제자연종료/Job0를검증하고별도진행한다. [v2 원실패](../validation/evidence/current22-readable-copy-api-run-v2-20261014/result.json).

최종v5 실제실행과Root독립검토통과: 같은v2 Rustbinary/provider2,014파일에서한글·emoji/예약CON 제목최종파일,기존대상거부·저장옵션불일치·사전취소·한글새이름복사·예약이름/변경된sourceID거부의10사례를확인했다. 모든합성영상내용은22byte이며원파일·collision sentinel·원도구와기존4FAIL을보존했다.9개trace에는exception이없었고private Job/desktop 자연0을확인했다. Root가actualraw/current669/2,014원SHA·binary/proof·생성/보존/부재파일을대조했다. [최종 실제 검토](../validation/evidence/current22-readable-copy-api-run-v5-20261014/root-review.json). WindowsfullID는실행된productionAPI의원receipt에서검증하며별도nativeID재조회완료로표현하지않는다.

원인분리: v3 diagnostic-init이legacy.NET경로조회중ArgumentException으로실패한것과v4 metadata-serialize의ConfigurationErrorsException/innerArgumentException을구분했다. v5는불필요한System.Web serializer만고정bounded JSON으로바꿨고자기소유짧은경로만정규화했다. 제품경로검증이나OS/보안설정은변경하지않았다. 실제provider/코덱/GUI복사/폴더대화상자·진행중취소는이실행범위외이며GUI배선검증을후속별도진행한다.
