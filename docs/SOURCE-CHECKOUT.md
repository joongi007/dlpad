# Source checkout and local validation evidence

GitHub에는 제품 소스, native resource·vendor·license, 정확한 Rust toolchain/Cargo.lock, 개발 문서와 CI 설정을 보관한다. 개발 전용 도구·cache·계정 설정·실제 사용자 자료·실행 로그/스크린샷·설치 ZIP·원시 검증 자료는 올리지 않고 원래 로컬 위치에 보존한다. `.gitignore`와 source export의 제외 정책을 따른다.

개발 기록의 `docs/validation/evidence/`, `.local-tools/` 및 validation 비Markdown 파일 링크는 이 소스 checkout에서 없을 수 있다. 그 링크나 과거 PASS 문구는 GitHub checkout이 그 시험을 실행했음을 뜻하지 않는다. 버전별 검증 범위와 미검증 사항은 해당 작업 기록에 남는다. 설치 파일의 공개 제공은 별도 release 단계다.

CI와 소스 checkout에서는 다음을 실행한다.

```sh
python3 scripts/check_docs.py --source-only
python3 scripts/test_check_docs.py
```

선택 모드는 source export의 명시적 제외 경로 링크만 따로 집계하고 runtime evidence를 검증하지 않았다고 출력한다. 저장소 밖 경로, 누락된 일반 문서·제품 소스·필수 문서·toolchain 계약은 계속 실패한다. 자동으로 누락을 허용하거나 기록을 PASS로 바꾸지 않는다.

로컬 증거가 있는 전체 작업 폴더의 기본 검사는 그대로 엄격하게 유지한다.

```sh
python3 scripts/check_docs.py
```

소스 export는 별도의 임시 복사본에 미포함 링크 안내를 만들며 현재 작업 폴더의 문서와 증거를 삭제·덮어쓰지 않는다. GitHub 저장소 연결과 Actions 실행은 [CI/CD](CICD.md)에서 추적한다.
