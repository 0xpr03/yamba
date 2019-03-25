package tech.yamba.db;

import org.jooq.codegen.DefaultGeneratorStrategy;
import org.jooq.codegen.GenerationTool;
import org.jooq.meta.Definition;
import org.jooq.meta.TableDefinition;
import org.jooq.meta.jaxb.Configuration;
import org.jooq.meta.jaxb.Database;
import org.jooq.meta.jaxb.Generate;
import org.jooq.meta.jaxb.Generator;
import org.jooq.meta.jaxb.Jdbc;
import org.jooq.meta.jaxb.Strategy;
import org.jooq.meta.jaxb.Target;


public class JooqGenerator {

	public static void main(String[] args) throws Exception {
		Configuration configuration = new Configuration()
				.withJdbc(new Jdbc()
						.withDriver("org.postgresql.Driver")
						.withUrl("jdbc:postgresql:postgres")
						.withUser("postgres")
						.withPassword("1234fuenf"))
				.withGenerator(new Generator()
						.withDatabase(new Database()
								.withName("org.jooq.meta.postgres.PostgresDatabase")
								.withExcludes("(spring_session|flyway|pg).*")
								.withInputSchema("public"))
						.withTarget(new Target()
								.withDirectory("src/main/java/")
								.withPackageName("tech.yamba.db.jooq")
								.withClean(true))
						.withStrategy(new Strategy()
								.withName(CustomGeneratorStrategy.class.getName()))
						.withGenerate(new Generate()
								.withDaos(true)
								.withPojos(true)
								.withPojosEqualsAndHashCode(true)
								.withFluentSetters(true)
								.withSpringAnnotations(true)
						));
		GenerationTool.generate(configuration);
	}


	public static class CustomGeneratorStrategy extends DefaultGeneratorStrategy {

		@Override public String getJavaClassName(Definition definition, Mode mode) {
			String generatedString = super.getJavaClassName(definition, mode);
			if (definition instanceof TableDefinition && mode == Mode.POJO) {
				return generatedString.replaceAll("ies$", "y").replaceAll("s$", "");
			}
			return generatedString;
		}
	}
}
